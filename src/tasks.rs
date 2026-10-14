//! Lecture et modification de l'etat des taches planifiees, via les cmdlets
//! PowerShell (Get/Disable/Enable-ScheduledTask) dont la sortie n'est PAS
//! localisee (noms d'enumeration .NET), contrairement a `schtasks`.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::process::Command;

/// Empeche l'ouverture d'une fenetre console a chaque appel PowerShell.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Etat d'une tache planifiee du point de vue de la telemetrie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// Tache desactivee (bon pour la confidentialite).
    Disabled,
    /// Tache active (Ready / Running).
    Enabled,
    /// Tache absente de ce Windows (certaines n'existent pas selon l'edition).
    Absent,
    /// Etat illisible.
    Unknown,
}

/// Separe un chemin complet en (dossier avec backslash final, nom de tache).
fn split(path: &str) -> (&str, &str) {
    match path.rfind('\\') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("\\", path),
    }
}

fn powershell(script: &str) -> Result<std::process::Output> {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("lancement de PowerShell")
}

/// Traduit l'etat .NET (non localise) renvoye par PowerShell en `TaskStatus`.
/// Une chaine vide signifie que la tache est absente de ce Windows.
fn parse_state(s: &str) -> TaskStatus {
    if s.is_empty() {
        TaskStatus::Absent
    } else if s.eq_ignore_ascii_case("Disabled") {
        TaskStatus::Disabled
    } else {
        // Ready, Running, etc.
        TaskStatus::Enabled
    }
}

/// Lit l'etat courant d'une seule tache planifiee (utilise a la sauvegarde).
pub fn status(path: &str) -> TaskStatus {
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; (Get-ScheduledTask -TaskPath '{folder}' -TaskName '{name}').State"
    );
    let out = match powershell(&script) {
        Ok(o) => o,
        Err(_) => return TaskStatus::Unknown,
    };
    parse_state(String::from_utf8_lossy(&out.stdout).trim())
}

/// Lit l'etat de plusieurs taches en **un seul** lancement de PowerShell.
///
/// Evite un demarrage de processus par tache (plusieurs centaines de ms chacun)
/// et donc le micro-gel de l'interface lors de l'actualisation des etats.
/// Les chemins absents de la reponse sont consideres `Unknown` par l'appelant.
pub fn statuses(paths: &[&str]) -> HashMap<String, TaskStatus> {
    let mut result = HashMap::new();
    if paths.is_empty() {
        return result;
    }
    // Liste de chemins litterale, entre apostrophes (aucun chemin ne contient
    // d'apostrophe). PowerShell construit une table etat-par-chemin complet,
    // puis emet « chemin<TAB>etat » pour chacun de nos chemins.
    let list = paths
        .iter()
        .map(|p| format!("'{p}'"))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; \
         $m=@{{}}; Get-ScheduledTask | ForEach-Object {{ $m[$_.TaskPath + $_.TaskName] = $_.State }}; \
         @({list}) | ForEach-Object {{ \"$_`t$($m[$_])\" }}"
    );
    let out = match powershell(&script) {
        Ok(o) => o,
        Err(_) => return result, // table vide : tout sera lu comme Unknown
    };
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if let Some((path, state)) = line.trim_end_matches('\r').split_once('\t') {
            result.insert(path.to_string(), parse_state(state.trim()));
        }
    }
    result
}

/// Desactive une tache (sans effet si absente). Best-effort.
pub fn disable(path: &str) -> Result<()> {
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; Disable-ScheduledTask -TaskPath '{folder}' -TaskName '{name}' | Out-Null"
    );
    powershell(&script)?;
    Ok(())
}

/// Reactive une tache (utilise par la restauration).
pub fn enable(path: &str) -> Result<()> {
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; Enable-ScheduledTask -TaskPath '{folder}' -TaskName '{name}' | Out-Null"
    );
    powershell(&script)?;
    Ok(())
}
