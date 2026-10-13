//! Lecture et modification de l'etat des taches planifiees, via les cmdlets
//! PowerShell (Get/Disable/Enable-ScheduledTask) dont la sortie n'est PAS
//! localisee (noms d'enumeration .NET), contrairement a `schtasks`.

use anyhow::{Context, Result};
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

/// Lit l'etat courant d'une tache planifiee.
pub fn status(path: &str) -> TaskStatus {
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='SilentlyContinue'; (Get-ScheduledTask -TaskPath '{folder}' -TaskName '{name}').State"
    );
    let out = match powershell(&script) {
        Ok(o) => o,
        Err(_) => return TaskStatus::Unknown,
    };
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        TaskStatus::Absent
    } else if s.eq_ignore_ascii_case("Disabled") {
        TaskStatus::Disabled
    } else {
        // Ready, Running, etc.
        TaskStatus::Enabled
    }
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
