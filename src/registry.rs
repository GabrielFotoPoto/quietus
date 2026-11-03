//! Acces bas niveau au registre Windows, enveloppant la crate `winreg`.

use crate::model::{Hive, RegData};
#[cfg(windows)]
use anyhow::Context;
use anyhow::{bail, Result};
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Verifie grossierement les droits administrateur en tentant d'ouvrir en ecriture
/// une cle machine. Sans admin, les reglages HKLM echoueront.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(r"SOFTWARE\Microsoft", KEY_SET_VALUE)
        .is_ok()
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    false
}

#[cfg(windows)]
fn root(hive: Hive) -> RegKey {
    match hive {
        Hive::Hklm => RegKey::predef(HKEY_LOCAL_MACHINE),
        Hive::Hkcu => RegKey::predef(HKEY_CURRENT_USER),
    }
}

/// Decode un DWORD (4 octets little-endian) de facon stricte.
pub fn decode_dword_bytes(bytes: &[u8], name: &str) -> Result<u32> {
    let arr: [u8; 4] = bytes.try_into().map_err(|_| {
        anyhow::anyhow!("taille DWORD invalide ({} octets) pour {name}", bytes.len())
    })?;
    Ok(u32::from_le_bytes(arr))
}

/// Lit une valeur. Renvoie `None` si la cle ou la valeur n'existe pas.
/// Renvoie une erreur explicite si le type de valeur n'est ni DWORD ni SZ/EXPAND_SZ
/// (evite d'ecraser un binaire ou QWORD avec une chaine vide dans une sauvegarde).
#[cfg(windows)]
pub fn read(hive: Hive, subkey: &str, name: &str) -> Result<Option<RegData>> {
    let key = match root(hive).open_subkey(subkey) {
        Ok(k) => k,
        Err(_) => return Ok(None),
    };
    match key.get_raw_value(name) {
        Ok(v) => match v.vtype {
            REG_DWORD => {
                let n = decode_dword_bytes(&v.bytes, name)?;
                Ok(Some(RegData::Dword(n)))
            }
            REG_SZ | REG_EXPAND_SZ => {
                let s: String = key
                    .get_value(name)
                    .with_context(|| format!("lecture SZ {subkey}\\{name}"))?;
                Ok(Some(RegData::Sz(s)))
            }
            other => {
                bail!("type de valeur registre non supporte ({other:?}) pour {subkey}\\{name}")
            }
        },
        Err(_) => Ok(None),
    }
}

#[cfg(not(windows))]
pub fn read(_hive: Hive, _subkey: &str, _name: &str) -> Result<Option<RegData>> {
    let _ = bail!("non supporte hors Windows");
    #[allow(unreachable_code)]
    Ok(None)
}

/// Ecrit une valeur, creant la cle au besoin.
#[cfg(windows)]
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

#[cfg(not(windows))]
pub fn write(_hive: Hive, _subkey: &str, _name: &str, _data: &RegData) -> Result<()> {
    Ok(())
}

/// Supprime une valeur (utilise pour annuler quand elle n'existait pas avant).
#[cfg(windows)]
pub fn delete_value(hive: Hive, subkey: &str, name: &str) -> Result<()> {
    if let Ok(key) = root(hive).open_subkey_with_flags(subkey, KEY_SET_VALUE) {
        let _ = key.delete_value(name);
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn delete_value(_hive: Hive, _subkey: &str, _name: &str) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_dword_bytes_accepts_4_bytes_and_rejects_invalid_sizes() {
        assert_eq!(decode_dword_bytes(&[4, 0, 0, 0], "Start").unwrap(), 4);
        assert!(decode_dword_bytes(&[], "Vide").is_err());
        assert!(decode_dword_bytes(&[1, 2], "TropCourt").is_err());
        assert!(decode_dword_bytes(&[1, 2, 3, 4, 5, 6, 7, 8], "Qword").is_err());
    }
}
