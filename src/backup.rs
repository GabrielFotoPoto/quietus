//! Sauvegarde et restauration : avant toute modification, on enregistre l'etat
//! anterieur (valeurs de registre ou etat des taches), pour pouvoir tout annuler.

use crate::model::{Action, Hive, RegData, Tweak};
use crate::registry;
use crate::tasks::{self, TaskStatus};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

/// Entree d'historique de sauvegarde pour l'interface de restauration.
#[derive(Clone, Debug)]
pub struct BackupEntry {
    pub path: PathBuf,
    pub filename: String,
    pub session: Session,
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
        created: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        tweaks,
    };
    let file = backup_dir()?.join(format!("backup-{}.json", now.format("%Y%m%d-%H%M%S-%3f")));
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
                    tasks::enable(path)?;
                }
            }
        }
    }
    Ok(())
}

/// Liste toutes les sessions de sauvegarde disponibles (de la plus recente a la plus ancienne).
pub fn list_sessions() -> Result<Vec<BackupEntry>> {
    let dir = backup_dir()?;
    list_sessions_from(&dir)
}

fn list_sessions_from(dir: &Path) -> Result<Vec<BackupEntry>> {
    let mut files: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|e| e == "json").unwrap_or(false))
        .collect();
    // Tri decroissant (plus recente en premier)
    files.sort_by(|a, b| b.cmp(a));

    let mut entries = Vec::with_capacity(files.len());
    for p in files {
        if let Ok(data) = std::fs::read(&p) {
            if let Ok(session) = serde_json::from_slice::<Session>(&data) {
                let filename = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                entries.push(BackupEntry {
                    path: p,
                    filename,
                    session,
                });
            }
        }
    }
    Ok(entries)
}

/// Supprime un fichier de sauvegarde en verifiant qu'il appartient bien au dossier de sauvegardes.
pub fn delete_backup(target: &Path) -> Result<()> {
    let dir = backup_dir()?;
    delete_backup_in(&dir, target)
}

fn delete_backup_in(dir: &Path, target: &Path) -> Result<()> {
    let canon_dir = std::fs::canonicalize(dir).context("dossier de sauvegarde invalide")?;
    let canon_target =
        std::fs::canonicalize(target).context("fichier de sauvegarde introuvable")?;
    if canon_target.parent() != Some(canon_dir.as_path()) {
        bail!("suppression refusee : le fichier est hors du dossier de sauvegardes");
    }
    if canon_target.extension().map(|e| e == "json") != Some(true) {
        bail!(
            "suppression refusee : seuls les fichiers .json de sauvegarde peuvent etre supprimes"
        );
    }
    let name = canon_target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    if !name.starts_with("backup-") {
        bail!("suppression refusee : nom de fichier inattendu ({name})");
    }
    std::fs::remove_file(canon_target).context("suppression du fichier de sauvegarde")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_sessions_returns_newest_first_and_delete_enforces_security_bounds() {
        let tmp = std::env::temp_dir().join(format!("quietus-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).expect("creation dossier temp");

        let s1 = Session {
            created: "2026-10-01 10:00:00".to_string(),
            tweaks: vec![TweakBackup {
                tweak_id: "fictif_1".to_string(),
                ops: vec![OpBackup::Registry {
                    hive: Hive::Hkcu,
                    subkey: r"Software\TestFictif".to_string(),
                    name: "Enabled".to_string(),
                    existed: false,
                    previous: None,
                }],
            }],
        };
        let s2 = Session {
            created: "2026-10-02 10:00:00".to_string(),
            tweaks: vec![TweakBackup {
                tweak_id: "fictif_2".to_string(),
                ops: vec![OpBackup::Task {
                    path: r"\TestFictif\Task".to_string(),
                    was_enabled: true,
                }],
            }],
        };

        let f1 = tmp.join("backup-20261001-100000.json");
        let f2 = tmp.join("backup-20261002-100000.json");
        std::fs::write(&f1, serde_json::to_vec_pretty(&s1).unwrap()).unwrap();
        std::fs::write(&f2, serde_json::to_vec_pretty(&s2).unwrap()).unwrap();

        let entries = list_sessions_from(&tmp).expect("lecture");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].session.tweaks[0].tweak_id, "fictif_2");
        assert_eq!(entries[1].session.tweaks[0].tweak_id, "fictif_1");

        // Securite : refus de supprimer un fichier qui ne commence pas par backup-
        let other_json = tmp.join("intrus.json");
        std::fs::write(&other_json, b"{}").unwrap();
        assert!(delete_backup_in(&tmp, &other_json).is_err());

        // Suppression legitime
        assert!(delete_backup_in(&tmp, &f2).is_ok());
        let entries_after = list_sessions_from(&tmp).expect("lecture apres suppression");
        assert_eq!(entries_after.len(), 1);
        assert_eq!(entries_after[0].session.tweaks[0].tweak_id, "fictif_1");

        // Nettoyage des donnees fictives de test
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
