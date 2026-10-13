//! Sauvegarde et restauration : avant toute modification, on enregistre l'etat
//! anterieur (valeurs de registre ou etat des taches), pour pouvoir tout annuler.

use crate::model::{Action, Hive, RegData, Tweak};
use crate::tasks::{self, TaskStatus};
use crate::registry;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Etat anterieur d'une operation elementaire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OpBackup {
    Registry {
        hive: Hive,
        subkey: String,
        name: String,
        /// La valeur existait-elle avant notre intervention ?
        existed: bool,
        previous: Option<RegData>,
    },
    Task {
        path: String,
        /// La tache etait-elle active avant notre intervention ?
        was_enabled: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TweakBackup {
    pub tweak_id: String,
    pub ops: Vec<OpBackup>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub created: String,
    pub tweaks: Vec<TweakBackup>,
}

/// Dossier de sauvegarde : %LOCALAPPDATA%\Quietus\backups
fn backup_dir() -> Result<PathBuf> {
    let base = std::env::var("LOCALAPPDATA").context("LOCALAPPDATA introuvable")?;
    let dir = PathBuf::from(base).join("Quietus").join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Capture l'etat courant des elements touches par un reglage, avant application.
pub fn capture(tweak: &Tweak) -> Result<TweakBackup> {
    let mut ops = Vec::new();
    match &tweak.action {
        Action::Registry(reg_ops) => {
            for op in reg_ops {
                let previous = registry::read(op.hive, op.subkey, op.name)?;
                ops.push(OpBackup::Registry {
                    hive: op.hive,
                    subkey: op.subkey.to_string(),
                    name: op.name.to_string(),
                    existed: previous.is_some(),
                    previous,
                });
            }
        }
        Action::ScheduledTasks(paths) => {
            for p in paths {
                ops.push(OpBackup::Task {
                    path: p.to_string(),
                    was_enabled: tasks::status(p) == TaskStatus::Enabled,
                });
            }
        }
    }
    Ok(TweakBackup {
        tweak_id: tweak.id.to_string(),
        ops,
    })
}

/// Enregistre une session de sauvegarde sur disque, renvoie le chemin du fichier.
pub fn save_session(tweaks: Vec<TweakBackup>) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let session = Session {
        created: now.to_rfc3339(),
        tweaks,
    };
    let file = backup_dir()?.join(format!("backup-{}.json", now.format("%Y%m%d-%H%M%S")));
    std::fs::write(&file, serde_json::to_vec_pretty(&session)?)?;
    Ok(file)
}

/// Restaure l'etat anterieur d'un reglage a partir de sa sauvegarde.
pub fn restore(tb: &TweakBackup) -> Result<()> {
    for op in &tb.ops {
        match op {
            OpBackup::Registry {
                hive,
                subkey,
                name,
                existed,
                previous,
            } => {
                if *existed {
                    if let Some(prev) = previous {
                        registry::write(*hive, subkey, name, prev)?;
                    }
                } else {
                    // La valeur n'existait pas : on la retire pour revenir a l'origine.
                    registry::delete_value(*hive, subkey, name)?;
                }
            }
            OpBackup::Task { path, was_enabled } => {
                // On ne reactive que ce qui etait actif ; sinon on laisse desactive/absent.
                if *was_enabled {
                    let _ = tasks::enable(path);
                }
            }
        }
    }
    Ok(())
}

/// Charge la derniere session de sauvegarde (pour « tout restaurer »).
pub fn load_latest() -> Result<Option<Session>> {
    let dir = backup_dir()?;
    let mut files: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    files.sort();
    match files.last() {
        Some(p) => {
            let data = std::fs::read(p)?;
            Ok(Some(serde_json::from_slice(&data)?))
        }
        None => Ok(None),
    }
}
