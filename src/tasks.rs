//! Lecture et modification de l'etat des taches planifiees, via les cmdlets
//! PowerShell (Get/Disable/Enable-ScheduledTask) dont la sortie n'est PAS
//! localisee (noms d'enumeration .NET), contrairement a `schtasks`.

use anyhow::{bail, Context, Result};
use std::collections::HashMap;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

/// Empeche l'ouverture d'une fenetre console a chaque appel PowerShell.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Caracteres interdits dans un chemin de tache planifiee (refus par defaut contre l'injection PowerShell).
const FORBIDDEN_CHARS: [char; 12] = [
    '\'', '"', '`', '$', ';', '|', '&', '<', '>', '\n', '\r', '\0',
];

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

/// Valide strictement un chemin de tache planifiee avant toute interpolation PowerShell.
pub fn validate_task_path(path: &str) -> Result<()> {
    if !path.starts_with('\\') || path.ends_with('\\') || path.len() < 2 {
        bail!("chemin de tache planifiee invalide : {path}");
    }
    if let Some(c) = path.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
        bail!("caractere interdit ({c:?}) dans le chemin de tache : {path}");
    }
    Ok(())
}

/// Retourne le chemin absolu vers `powershell.exe` dans System32 pour eviter
/// tout detournement via le `PATH` lors d'une execution avec droits administrateur.
fn powershell_executable() -> PathBuf {
    let root = std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    root.join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe")
}

/// Separe un chemin complet en (dossier avec backslash final, nom de tache).
fn split(path: &str) -> (&str, &str) {
    match path.rfind('\\') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("\\", path),
    }
}

fn powershell(script: &str) -> Result<std::process::Output> {
    let mut cmd = Command::new(powershell_executable());
    cmd.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        script,
    ]);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().context("lancement de PowerShell")
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
    if validate_task_path(path).is_err() {
        return TaskStatus::Unknown;
    }
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
    let mut valid_paths = Vec::with_capacity(paths.len());
    for &p in paths {
        if validate_task_path(p).is_ok() {
            valid_paths.push(format!("'{p}'"));
        }
    }
    if valid_paths.is_empty() {
        return result;
    }
    let list = valid_paths.join(",");
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

/// Desactive une tache (sans effet si absente, erreur explicite si verrouillee par l'OS).
pub fn disable(path: &str) -> Result<()> {
    validate_task_path(path)?;
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         $t = Get-ScheduledTask -TaskPath '{folder}' -TaskName '{name}' -ErrorAction SilentlyContinue; \
         if ($null -ne $t -and $t.State -ne 'Disabled') {{ \
             Disable-ScheduledTask -InputObject $t -ErrorAction Stop | Out-Null \
         }}"
    );
    let out = powershell(&script)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        bail!("echec de desactivation de la tache {path} : {err}");
    }
    Ok(())
}

/// Reactive une tache (utilise par la restauration).
pub fn enable(path: &str) -> Result<()> {
    validate_task_path(path)?;
    let (folder, name) = split(path);
    let script = format!(
        "$ErrorActionPreference='Stop'; \
         $t = Get-ScheduledTask -TaskPath '{folder}' -TaskName '{name}' -ErrorAction SilentlyContinue; \
         if ($null -ne $t -and $t.State -eq 'Disabled') {{ \
             Enable-ScheduledTask -InputObject $t -ErrorAction Stop | Out-Null \
         }}"
    );
    let out = powershell(&script)?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        bail!("echec de reactivation de la tache {path} : {err}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_separates_folder_and_task_name() {
        let (folder, name) = split(r"\Microsoft\Windows\Autochk\Proxy");
        assert_eq!(folder, r"\Microsoft\Windows\Autochk\");
        assert_eq!(name, "Proxy");
    }

    #[test]
    fn split_handles_root_or_bare_task_name() {
        let (folder, name) = split(r"\RootTask");
        assert_eq!(folder, r"\");
        assert_eq!(name, "RootTask");

        let (folder_bare, name_bare) = split("BareTask");
        assert_eq!(folder_bare, r"\");
        assert_eq!(name_bare, "BareTask");
    }

    #[test]
    fn parse_state_maps_dotnet_states_correctly() {
        assert_eq!(parse_state(""), TaskStatus::Absent);
        assert_eq!(parse_state("Disabled"), TaskStatus::Disabled);
        assert_eq!(parse_state("disabled"), TaskStatus::Disabled);
        assert_eq!(parse_state("Ready"), TaskStatus::Enabled);
        assert_eq!(parse_state("Running"), TaskStatus::Enabled);
    }

    #[test]
    fn statuses_returns_empty_map_for_empty_slice() {
        let map = statuses(&[]);
        assert!(map.is_empty());
    }

    #[test]
    fn security_validate_task_path_rejects_injection_and_malformed_paths() {
        assert!(validate_task_path(r"\Microsoft\Windows\Autochk\Proxy").is_ok());

        for bad in [
            "",
            r"\",
            r"Microsoft\SansBackslashInitial",
            r"\Microsoft\DossierSansNom\",
            r"\Microsoft\Task'; Start-Process calc; '",
            r"\Microsoft\Task$(whoami)",
            "\\Microsoft\\Task\nCalc",
            r"\Microsoft\Task|Out-File",
            r"\Microsoft\Task&whoami",
        ] {
            assert!(
                validate_task_path(bad).is_err(),
                "le chemin malveillant ou malforme aurait du etre rejete : {bad}"
            );
        }
    }

    #[test]
    fn security_powershell_executable_uses_absolute_system32_path() {
        let exe = powershell_executable();
        let s = exe.to_string_lossy();
        assert!(
            s.ends_with(r"System32\WindowsPowerShell\v1.0\powershell.exe")
                || s.ends_with("System32/WindowsPowerShell/v1.0/powershell.exe")
        );
    }
}
