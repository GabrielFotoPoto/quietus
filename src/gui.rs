//! Interface graphique (egui / eframe). Branche sur le meme coeur que la logique
//! metier : catalogue, detection, application et restauration restent inchanges.

use crate::model::{Category, Tweak, TweakState};
use crate::{backup, registry, tasks, tweaks};
use eframe::egui;
use egui::Color32;

const ORANGE: Color32 = Color32::from_rgb(210, 170, 60);

struct Row {
    tweak: Tweak,
    state: TweakState,
    selected: bool,
}

pub struct QuietusApp {
    rows: Vec<Row>,
    focused: usize,
    status: String,
    elevated: bool,
    /// La fenetre de confirmation avant application est-elle ouverte ?
    confirm_open: bool,
    /// Creer un point de restauration systeme Windows avant d'appliquer ?
    create_restore_point: bool,
    /// La fenetre de choix de sauvegarde a restaurer est-elle ouverte ?
    restore_open: bool,
    /// Liste des sessions de sauvegarde disponibles.
    backups_list: Vec<backup::BackupEntry>,
    /// Index de la session selectionnee dans `backups_list`.
    selected_backup: usize,
}

impl QuietusApp {
    pub fn new() -> Self {
        let states = tweaks::task_states();
        let rows = tweaks::catalog()
            .into_iter()
            .map(|t| {
                let state = tweaks::detect(&t, &states);
                Row {
                    tweak: t,
                    state,
                    selected: false,
                }
            })
            .collect();
        Self {
            rows,
            focused: 0,
            status: "Cochez des reglages, puis ? Appliquer la selection ?.".into(),
            elevated: registry::is_elevated(),
            confirm_open: false,
            create_restore_point: true,
            restore_open: false,
            backups_list: Vec::new(),
            selected_backup: 0,
        }
    }

    fn refresh(&mut self) {
        let states = tweaks::task_states();
        for r in &mut self.rows {
            r.state = tweaks::detect(&r.tweak, &states);
        }
        self.status = "Etats actualises.".into();
    }

    fn set_category(&mut self, cat: Category, selected: bool) {
        for r in &mut self.rows {
            if r.tweak.category == cat {
                r.selected = selected;
            }
        }
    }

    fn selected_count(&self) -> usize {
        self.rows.iter().filter(|r| r.selected).count()
    }

    fn tweak_name_by_id<'a>(&'a self, id: &'a str) -> &'a str {
        self.rows
            .iter()
            .find(|r| r.tweak.id == id)
            .map(|r| r.tweak.name)
            .unwrap_or(id)
    }

    fn apply_selected(&mut self) {
        if self.selected_count() == 0 {
            self.status = "Rien de selectionne.".into();
            return;
        }
        if self.create_restore_point {
            if let Err(e) = tasks::create_restore_point() {
                self.status = format!(
                    "Point de restauration impossible (decochez l'option ou activez la Protection du systeme) : {e}"
                );
                return;
            }
        }

        let mut backups = Vec::new();
        let mut applied = Vec::new();
        let mut reboot = false;
        for i in 0..self.rows.len() {
            if !self.rows[i].selected {
                continue;
            }
            match backup::capture(&self.rows[i].tweak) {
                Ok(b) => backups.push(b),
                Err(e) => {
                    self.status =
                        format!("Sauvegarde impossible pour {} : {e}", self.rows[i].tweak.id);
                    return;
                }
            }
            if let Err(e) = tweaks::apply(&self.rows[i].tweak) {
                self.status = format!("Echec sur {} (droits admin ?) : {e}", self.rows[i].tweak.id);
                return;
            }
            reboot |= self.rows[i].tweak.requires_reboot;
            applied.push(i);
        }
        let count = applied.len();
        // Re-detection groupee : un seul appel PowerShell pour toutes les taches.
        let states = tweaks::task_states();
        for &i in &applied {
            self.rows[i].state = tweaks::detect(&self.rows[i].tweak, &states);
            self.rows[i].selected = false;
        }
        match backup::save_session(backups) {
            Ok(path) => {
                self.status = format!(
                    "{count} reglage(s) applique(s). Sauvegarde : {}{}",
                    path.display(),
                    if reboot {
                        "  -  redemarrage conseille."
                    } else {
                        ""
                    }
                );
            }
            Err(e) => self.status = format!("Applique, mais sauvegarde non ecrite : {e}"),
        }
    }

    fn open_restore_window(&mut self) {
        match backup::list_sessions() {
            Ok(list) => {
                if list.is_empty() {
                    self.status = "Aucune sauvegarde a restaurer.".into();
                } else {
                    self.backups_list = list;
                    self.selected_backup = 0;
                    self.restore_open = true;
                }
            }
            Err(e) => self.status = format!("Lecture des sauvegardes impossible : {e}"),
        }
    }

    fn restore_selected_session(&mut self) {
        let Some(entry) = self.backups_list.get(self.selected_backup).cloned() else {
            return;
        };
        let mut ok = 0;
        let mut errs = Vec::new();
        for tb in &entry.session.tweaks {
            match backup::restore(tb) {
                Ok(()) => ok += 1,
                Err(e) => errs.push(format!("{} ({e})", tb.tweak_id)),
            }
        }
        self.refresh();
        if errs.is_empty() {
            self.status = format!(
                "Restauration ({}) : {ok} reglage(s) remis a leur etat anterieur.",
                entry.session.created
            );
        } else {
            self.status = format!(
                "Restauration partielle ({ok} ok) ; erreurs : {}",
                errs.join(", ")
            );
        }
    }

    fn delete_selected_backup(&mut self) {
        let Some(entry) = self.backups_list.get(self.selected_backup).cloned() else {
            return;
        };
        match backup::delete_backup(&entry.path) {
            Ok(()) => {
                self.status = format!("Sauvegarde supprimee : {}", entry.filename);
                self.backups_list.remove(self.selected_backup);
                if self.backups_list.is_empty() {
                    self.restore_open = false;
                } else if self.selected_backup >= self.backups_list.len() {
                    self.selected_backup = self.backups_list.len() - 1;
                }
            }
            Err(e) => self.status = format!("Suppression impossible : {e}"),
        }
    }

    fn confirm_window(&mut self, ctx: &egui::Context) {
        if !self.confirm_open {
            return;
        }
        let selected: Vec<&'static str> = self
            .rows
            .iter()
            .filter(|r| r.selected)
            .map(|r| r.tweak.name)
            .collect();
        let reboot = self
            .rows
            .iter()
            .any(|r| r.selected && r.tweak.requires_reboot);
        let mut do_apply = false;

        egui::Window::new("Confirmer l'application")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} reglage(s) vont etre desactives :",
                    selected.len()
                ));
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for n in &selected {
                            ui.label(format!("?  {n}"));
                        }
                    });
                if reboot {
                    ui.add_space(4.0);
                    ui.colored_label(ORANGE, "Un redemarrage sera conseille apres application.");
                }
                ui.separator();
                ui.checkbox(
                    &mut self.create_restore_point,
                    "Creer un point de restauration systeme Windows avant d'agir",
                );
                ui.small("(en plus de la sauvegarde JSON locale automatique)");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Confirmer").clicked() {
                        do_apply = true;
                    }
                    if ui.button("Annuler").clicked() {
                        self.confirm_open = false;
                    }
                });
            });

        if do_apply {
            self.apply_selected();
            self.confirm_open = false;
        }
    }

    fn restore_window(&mut self, ctx: &egui::Context) {
        if !self.restore_open {
            return;
        }
        let mut do_restore = false;
        let mut do_delete = false;
        let mut do_close = false;

        egui::Window::new("Restaurer une sauvegarde")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label("Choisissez la session a restaurer :");
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("backups_scroll")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (idx, entry) in self.backups_list.iter().enumerate() {
                            let suffix = if idx == 0 { " (plus recente)" } else { "" };
                            let label = format!(
                                "{} - {} reglage(s){suffix}",
                                entry.session.created,
                                entry.session.tweaks.len()
                            );
                            ui.radio_value(&mut self.selected_backup, idx, label);
                        }
                    });

                ui.separator();
                if let Some(entry) = self.backups_list.get(self.selected_backup) {
                    ui.label("Reglages contenus dans cette sauvegarde :");
                    ui.add_space(4.0);
                    egui::ScrollArea::vertical()
                        .id_salt("tweaks_preview_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for tb in &entry.session.tweaks {
                                let name = self.tweak_name_by_id(&tb.tweak_id);
                                ui.label(format!("?  {} ({})", tb.tweak_id, name));
                            }
                        });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Restaurer cette session").clicked() {
                        do_restore = true;
                    }
                    if ui.button("Supprimer ce fichier").clicked() {
                        do_delete = true;
                    }
                    if ui.button("Fermer").clicked() {
                        do_close = true;
                    }
                });
            });

        if do_restore {
            self.restore_selected_session();
            self.restore_open = false;
        } else if do_delete {
            self.delete_selected_backup();
        } else if do_close {
            self.restore_open = false;
        }
    }
}

fn state_badge(ui: &mut egui::Ui, s: TweakState) {
    let (txt, col) = match s {
        TweakState::Disabled => ("desactive", Color32::from_rgb(80, 180, 80)),
        TweakState::Enabled => ("actif", ORANGE),
        TweakState::Unknown => ("inconnu", Color32::GRAY),
    };
    ui.label(egui::RichText::new(txt).color(col).monospace());
}

impl eframe::App for QuietusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("Quietus");
                ui.label("- telemetrie Windows 11 sous votre controle");
            });
            if !self.elevated {
                ui.colored_label(
                    Color32::from_rgb(220, 80, 80),
                    "Droits administrateur manquants : les reglages ? machine ? echoueront. Relancez en administrateur.",
                );
            }
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let n = self.selected_count();
                if ui.button(format!("Appliquer la selection ({n})")).clicked() {
                    if n > 0 {
                        self.confirm_open = true;
                    } else {
                        self.status = "Rien de selectionne.".into();
                    }
                }
                if ui.button("Restaurer...").clicked() {
                    self.open_restore_window();
                }
                if ui.button("Rafraichir les etats").clicked() {
                    self.refresh();
                }
            });
            ui.add_space(2.0);
            ui.label(&self.status);
            ui.add_space(4.0);
        });

        egui::SidePanel::right("detail")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                if let Some(r) = self.rows.get(self.focused) {
                    ui.heading(r.tweak.name);
                    ui.separator();
                    ui.label(format!("Categorie : {}", r.tweak.category.label()));
                    ui.label(format!("Risque : {}", r.tweak.risk.label()));
                    ui.horizontal(|ui| {
                        ui.label("Etat :");
                        state_badge(ui, r.state);
                    });
                    if r.tweak.requires_reboot {
                        ui.colored_label(ORANGE, "Redemarrage requis apres application.");
                    }
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Ce que ca fait").strong());
                    ui.label(r.tweak.description);
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Si desactive").strong());
                    ui.label(r.tweak.impact);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for cat in Category::ALL {
                    let has_rows = self.rows.iter().any(|r| r.tweak.category == cat);
                    if !has_rows {
                        continue;
                    }
                    egui::CollapsingHeader::new(egui::RichText::new(cat.label()).strong())
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.small_button("Tout cocher").clicked() {
                                    self.set_category(cat, true);
                                }
                                if ui.small_button("Tout decocher").clicked() {
                                    self.set_category(cat, false);
                                }
                            });
                            for i in 0..self.rows.len() {
                                if self.rows[i].tweak.category != cat {
                                    continue;
                                }
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut self.rows[i].selected, "");
                                    state_badge(ui, self.rows[i].state);
                                    let name = self.rows[i].tweak.name;
                                    if ui.selectable_label(self.focused == i, name).clicked() {
                                        self.focused = i;
                                    }
                                });
                            }
                            ui.add_space(4.0);
                        });
                }
            });
        });

        self.confirm_window(ctx);
        self.restore_window(ctx);
    }
}
