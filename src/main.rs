//! Quietus : gestionnaire de telemetrie Windows 11, controle par l'utilisateur.
//! Tout reglage applique est sauvegarde avant modification et reversible.

mod backup;
mod model;
mod registry;
mod tweaks;
mod ui;

use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_SET_VALUE};
use winreg::RegKey;

/// Verifie grossierement les droits administrateur en tentant d'ouvrir en ecriture
/// une cle machine. Sert d'avertissement : sans admin, les reglages HKLM echoueront.
fn is_elevated() -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft", KEY_SET_VALUE)
        .is_ok()
}

fn main() -> anyhow::Result<()> {
    if !is_elevated() {
        eprintln!(
            "Attention : l'outil ne semble pas disposer des droits administrateur.\n\
             Les reglages « machine » (HKLM) echoueront. Relance en tant qu'administrateur.\n"
        );
    }
    ui::run()
}
