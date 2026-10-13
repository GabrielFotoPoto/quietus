//! Catalogue des reglages de confidentialite (Windows 11), et logique
//! de detection / application. Tout est reversible via le module `backup`.

use crate::model::*;
use crate::registry;
use crate::tasks::{self, TaskStatus};
use anyhow::Result;

fn dword(hive: Hive, subkey: &'static str, name: &'static str, disable: u32) -> RegOp {
    RegOp {
        hive,
        subkey,
        name,
        disable: RegData::Dword(disable),
    }
}

fn reg(ops: Vec<RegOp>) -> Action {
    Action::Registry(ops)
}

/// Construit le catalogue complet des reglages.
pub fn catalog() -> Vec<Tweak> {
    use Category::*;
    use Hive::*;
    use Risk::*;

    vec![
        Tweak {
            id: "allow_telemetry",
            name: "Niveau de donnees de diagnostic (AllowTelemetry)",
            category: TelemetrieCoeur,
            risk: Sur,
            description: "Regle par strategie le niveau de donnees de diagnostic envoyees a Microsoft.",
            impact: "Reduit la collecte au minimum. Sur une edition Pro, Windows maintient un socle « Requis » non supprimable par cette seule cle.",
            requires_reboot: false,
            action: reg(vec![dword(Hklm, r"SOFTWARE\Policies\Microsoft\Windows\DataCollection", "AllowTelemetry", 0)]),
        },
        Tweak {
            id: "diagtrack_service",
            name: "Service DiagTrack (Connected User Experiences and Telemetry)",
            category: TelemetrieCoeur,
            risk: Modere,
            description: "Service Windows qui collecte et transmet les donnees de diagnostic.",
            impact: "Coupe la remontee de telemetrie a la source. Sans effet notable sur l'usage courant. Redemarrage requis.",
            requires_reboot: true,
            action: reg(vec![dword(Hklm, r"SYSTEM\CurrentControlSet\Services\DiagTrack", "Start", 4)]),
        },
        Tweak {
            id: "dmwappush_service",
            name: "Service dmwappushservice (routage de messages WAP)",
            category: TelemetrieCoeur,
            risk: Modere,
            description: "Service lie a la remontee de donnees ; inutile hors gestion d'appareils mobiles.",
            impact: "Aucun effet pour un poste de bureau classique. Redemarrage requis.",
            requires_reboot: true,
            action: reg(vec![dword(Hklm, r"SYSTEM\CurrentControlSet\Services\dmwappushservice", "Start", 4)]),
        },
        Tweak {
            id: "ceip_tasks",
            name: "Taches du programme d'amelioration (CEIP)",
            category: TelemetrieCoeur,
            risk: Modere,
            description: "Taches planifiees qui collectent et envoient des donnees d'usage (Consolidator, UsbCeip).",
            impact: "Fin de la collecte CEIP planifiee. Aucun effet fonctionnel. Taches absentes ignorees.",
            requires_reboot: false,
            action: Action::ScheduledTasks(vec![
                r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
                r"\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
                r"\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipTask",
                r"\Microsoft\Windows\Autochk\Proxy",
            ]),
        },
        Tweak {
            id: "appraiser_tasks",
            name: "Taches d'evaluation de compatibilite",
            category: TelemetrieCoeur,
            risk: Modere,
            description: "Taches qui analysent le systeme et les apps pour la telemetrie de compatibilite.",
            impact: "Stoppe l'analyse de compatibilite planifiee. Sans effet sur l'usage courant.",
            requires_reboot: false,
            action: Action::ScheduledTasks(vec![
                // Nom classique (Win10 / anciens Win11) ...
                r"\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser",
                // ... et nom renomme sur les Win11 recents (build 26xxx).
                r"\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser Exp",
                r"\Microsoft\Windows\Application Experience\ProgramDataUpdater",
                r"\Microsoft\Windows\Application Experience\StartupAppTask",
            ]),
        },
        Tweak {
            id: "error_reporting",
            name: "Rapports d'erreurs Windows (WER)",
            category: RapportsEtRetours,
            risk: Modere,
            description: "Envoie a Microsoft des rapports lors des plantages d'applications ou du systeme.",
            impact: "Les plantages ne sont plus transmis. Le diagnostic local reste possible via l'Observateur d'evenements.",
            requires_reboot: false,
            action: reg(vec![dword(Hklm, r"SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting", "Disabled", 1)]),
        },
        Tweak {
            id: "feedback_frequency",
            name: "Demandes de retour d'experience (feedback)",
            category: RapportsEtRetours,
            risk: Sur,
            description: "Frequence a laquelle Windows sollicite ton avis via des fenetres de feedback.",
            impact: "Supprime les demandes de feedback. Aucun effet fonctionnel.",
            requires_reboot: false,
            action: reg(vec![dword(Hkcu, r"Software\Microsoft\Siuf\Rules", "NumberOfSIUFInPeriod", 0)]),
        },
        Tweak {
            id: "feedback_tasks",
            name: "Taches de retour d'experience (SIUF / DmClient)",
            category: RapportsEtRetours,
            risk: Sur,
            description: "Taches planifiees liees aux fenetres de feedback et a leur telechargement.",
            impact: "Fin des taches de feedback planifiees. Aucun effet fonctionnel.",
            requires_reboot: false,
            action: Action::ScheduledTasks(vec![
                r"\Microsoft\Windows\Feedback\Siuf\DmClient",
                r"\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload",
            ]),
        },
        Tweak {
            id: "advertising_id",
            name: "Identifiant de publicite",
            category: PubliciteEtPerso,
            risk: Sur,
            description: "Identifiant unique utilise pour personnaliser les publicites entre applications.",
            impact: "Les apps ne peuvent plus te pister via cet identifiant. Les pubs restent presentes mais non personnalisees.",
            requires_reboot: false,
            action: reg(vec![dword(Hkcu, r"SOFTWARE\Microsoft\Windows\CurrentVersion\AdvertisingInfo", "Enabled", 0)]),
        },
        Tweak {
            id: "tailored_experiences",
            name: "Experiences personnalisees basees sur les diagnostics",
            category: PubliciteEtPerso,
            risk: Sur,
            description: "Utilise tes donnees de diagnostic pour des suggestions et publicites ciblees.",
            impact: "Fin des suggestions personnalisees basees sur la telemetrie. Aucun effet fonctionnel.",
            requires_reboot: false,
            action: reg(vec![dword(Hkcu, r"Software\Microsoft\Windows\CurrentVersion\Privacy", "TailoredExperiencesWithDiagnosticDataEnabled", 0)]),
        },
        Tweak {
            id: "start_suggestions",
            name: "Suggestions et contenu propose (menu Demarrer / Explorateur)",
            category: PubliciteEtPerso,
            risk: Sur,
            description: "Affiche des applications suggerees, du contenu promotionnel et des astuces.",
            impact: "Supprime les suggestions et le contenu promotionnel. Interface plus sobre.",
            requires_reboot: false,
            action: reg(vec![
                dword(Hkcu, r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SystemPaneSuggestionsEnabled", 0),
                dword(Hkcu, r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager", "SilentInstalledAppsEnabled", 0),
            ]),
        },
        Tweak {
            id: "activity_history",
            name: "Historique d'activite (Timeline)",
            category: ViePriveeUsage,
            risk: Sur,
            description: "Collecte et televerse l'historique de tes activites pour la fonction Timeline.",
            impact: "L'historique n'est plus collecte ni televerse. La Timeline cesse de se remplir.",
            requires_reboot: false,
            action: reg(vec![
                dword(Hklm, r"SOFTWARE\Policies\Microsoft\Windows\System", "PublishUserActivities", 0),
                dword(Hklm, r"SOFTWARE\Policies\Microsoft\Windows\System", "UploadUserActivities", 0),
            ]),
        },
        Tweak {
            id: "app_launch_tracking",
            name: "Suivi des lancements d'applications",
            category: ViePriveeUsage,
            risk: Sur,
            description: "Suit les applications que tu ouvres pour « ameliorer les resultats de recherche ».",
            impact: "Le suivi cesse. Le menu Demarrer n'affiche plus les apps les plus utilisees.",
            requires_reboot: false,
            action: reg(vec![dword(Hkcu, r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced", "Start_TrackProgs", 0)]),
        },
        Tweak {
            id: "inking_typing",
            name: "Personnalisation saisie et ecriture manuscrite",
            category: ViePriveeUsage,
            risk: Sur,
            description: "Envoie des echantillons de ta frappe et de ton ecriture pour personnaliser la saisie.",
            impact: "Fin de l'envoi d'echantillons. Les suggestions de saisie personnalisees sont desactivees.",
            requires_reboot: false,
            action: reg(vec![
                dword(Hkcu, r"Software\Microsoft\InputPersonalization", "RestrictImplicitTextCollection", 1),
                dword(Hkcu, r"Software\Microsoft\InputPersonalization", "RestrictImplicitInkCollection", 1),
            ]),
        },
        Tweak {
            id: "location",
            name: "Service de localisation (global)",
            category: Localisation,
            risk: Modere,
            description: "Autorise Windows et les applications a determiner ta position geographique.",
            impact: "Desactive la localisation pour tout le systeme. Meteo, cartes et « Localiser mon appareil » ne fonctionneront plus.",
            requires_reboot: false,
            action: reg(vec![RegOp {
                hive: Hklm,
                subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location",
                name: "Value",
                disable: RegData::Sz("Deny".to_string()),
            }]),
        },
    ]
}

/// Determine si un reglage est deja applique (telemetrie desactivee).
pub fn detect(tweak: &Tweak) -> TweakState {
    match &tweak.action {
        Action::Registry(ops) => {
            let mut all_match = true;
            for op in ops {
                match registry::read(op.hive, op.subkey, op.name) {
                    Ok(Some(cur)) if cur == op.disable => {}
                    Ok(_) => all_match = false,
                    Err(_) => return TweakState::Unknown,
                }
            }
            if all_match {
                TweakState::Disabled
            } else {
                TweakState::Enabled
            }
        }
        Action::ScheduledTasks(paths) => {
            let mut all_off = true;
            for p in paths {
                match tasks::status(p) {
                    // Desactivee ou absente : rien a remonter pour cette tache.
                    TaskStatus::Disabled | TaskStatus::Absent => {}
                    TaskStatus::Enabled => all_off = false,
                    TaskStatus::Unknown => return TweakState::Unknown,
                }
            }
            if all_off {
                TweakState::Disabled
            } else {
                TweakState::Enabled
            }
        }
    }
}

/// Applique un reglage (desactive la telemetrie correspondante).
pub fn apply(tweak: &Tweak) -> Result<()> {
    match &tweak.action {
        Action::Registry(ops) => {
            for op in ops {
                registry::write(op.hive, op.subkey, op.name, &op.disable)?;
            }
        }
        Action::ScheduledTasks(paths) => {
            for p in paths {
                // Best-effort : une tache absente est ignoree silencieusement.
                let _ = tasks::disable(p);
            }
        }
    }
    Ok(())
}
