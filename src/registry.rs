//! Acces bas niveau au registre Windows, enveloppant la crate `winreg`.

use crate::model::{Hive, RegData};
use anyhow::{Context, Result};
use winreg::enums::*;
use winreg::RegKey;

fn root(hive: Hive) -> RegKey {
    match hive {
        Hive::Hklm => RegKey::predef(HKEY_LOCAL_MACHINE),
        Hive::Hkcu => RegKey::predef(HKEY_CURRENT_USER),
    }
}

/// Lit une valeur. Renvoie `None` si la cle ou la valeur n'existe pas.
pub fn read(hive: Hive, subkey: &str, name: &str) -> Result<Option<RegData>> {
    let key = match root(hive).open_subkey(subkey) {
        Ok(k) => k,
        Err(_) => return Ok(None),
    };
    match key.get_raw_value(name) {
        Ok(v) => match v.vtype {
            REG_DWORD => {
                let n = u32::from_le_bytes(
                    v.bytes
                        .get(0..4)
                        .and_then(|s| s.try_into().ok())
                        .unwrap_or([0, 0, 0, 0]),
                );
                Ok(Some(RegData::Dword(n)))
            }
            _ => {
                let s: String = key.get_value(name).unwrap_or_default();
                Ok(Some(RegData::Sz(s)))
            }
        },
        Err(_) => Ok(None),
    }
}

/// Ecrit une valeur, creant la cle au besoin.
pub fn write(hive: Hive, subkey: &str, name: &str, data: &RegData) -> Result<()> {
    let (key, _) = root(hive)
        .create_subkey(subkey)
        .with_context(|| format!("creation/ouverture de la cle {subkey}"))?;
    match data {
        RegData::Dword(n) => key
            .set_value(name, n)
            .with_context(|| format!("ecriture DWORD {name}"))?,
        RegData::Sz(s) => key
            .set_value(name, s)
            .with_context(|| format!("ecriture SZ {name}"))?,
    }
    Ok(())
}

/// Supprime une valeur (utilise pour annuler quand elle n'existait pas avant).
pub fn delete_value(hive: Hive, subkey: &str, name: &str) -> Result<()> {
    if let Ok(key) = root(hive).open_subkey_with_flags(subkey, KEY_SET_VALUE) {
        let _ = key.delete_value(name);
    }
    Ok(())
}
