//! Modele de donnees : un « reglage » (Tweak) decrit une action de
//! confidentialite, son explication, son risque, et comment la lire / appliquer / annuler.

use serde::{Deserialize, Serialize};

/// Ruche de registre visee.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hive {
    /// HKEY_LOCAL_MACHINE (reglages machine, requiert admin).
    Hklm,
    /// HKEY_CURRENT_USER (reglages de l'utilisateur courant).
    Hkcu,
}

/// Donnee de registre geree en v1 (la grande majorite des tweaks sont des DWORD).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegData {
    Dword(u32),
    Sz(String),
}

/// Categorie d'affichage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    TelemetrieCoeur,
    RapportsEtRetours,
    PubliciteEtPerso,
    ViePriveeUsage,
    Localisation,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::TelemetrieCoeur => "Telemetrie coeur",
            Category::RapportsEtRetours => "Rapports & retours",
            Category::PubliciteEtPerso => "Publicite & personnalisation",
            Category::ViePriveeUsage => "Vie privee d'usage",
            Category::Localisation => "Localisation",
        }
    }
}

/// Niveau de risque indicatif.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Risk {
    Sur,
    Modere,
}

impl Risk {
    pub fn label(self) -> &'static str {
        match self {
            Risk::Sur => "Sur",
            Risk::Modere => "Modere",
        }
    }
}

/// Une operation de registre elementaire : poser `disable` desactive la telemetrie.
#[derive(Clone, Debug)]
pub struct RegOp {
    pub hive: Hive,
    pub subkey: &'static str,
    pub name: &'static str,
    /// Valeur a ecrire pour DESACTIVER la fonction de telemetrie.
    pub disable: RegData,
}

/// Etat courant d'un reglage, calcule en lisant le registre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TweakState {
    /// La telemetrie est deja desactivee (toutes les valeurs « disable » sont en place).
    Disabled,
    /// La telemetrie est active (au moins une valeur ne correspond pas).
    Enabled,
    /// Impossible de lire l'etat (cle inaccessible, etc.).
    Unknown,
}

/// Definition complete d'un reglage.
#[derive(Clone, Debug)]
pub struct Tweak {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub risk: Risk,
    /// Ce que fait la fonction ciblee.
    pub description: &'static str,
    /// Ce que sa desactivation implique concretement pour l'utilisateur.
    pub impact: &'static str,
    pub requires_reboot: bool,
    pub ops: Vec<RegOp>,
}
