//! Interface graphique (egui / eframe). Branche sur le meme coeur que la logique
//! metier : catalogue, detection, application et restauration restent inchanges.

use crate::model::{Category, Tweak, TweakState};
use crate::{backup, registry, tweaks};
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
}

impl QuietusApp {
    pub fn new() -> Self {
        let rows = tweaks::catalog()
            .into_iter()
            .map(|t| {
                let state = tweaks::detect(&t);
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
            status: "Cochez des reglages, puis « Appliquer la selection ».".into(),
            elevated: registry::is_elevated(),
            confirm_open: false,
        }
    }

    fn refresh(&mut self) {
        for r in &mut self.rows {
            r.state = tweaks::detect(&r.tweak);
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

    fn apply_selected(&mut self) {
        let mut backups = Vec::new();
        let mut count = 0;
        let mut reboot = false;
        for row in self.rows.iter_mut().filter(|r| r.selected) {
            match backup::capture(&row.tweak) {
                Ok(b) => backups.push(b),
                Err(e) => {
                    self.status = format!("Sauvegarde impossible pour {} : {e}", row.tweak.id);
                    return;
                }
            }
            if let Err(e) = tweaks::apply(&row.tweak) {
                self.status = format!("Echec sur {} (droits admin ?) : {e}", row.tweak.id);
                return;
            }
            row.state = tweaks::detect(&row.tweak);
            row.selected = false;
            reboot |= row.tweak.requires_reboot;
            count += 1;
        }
        if count == 0 {
            self.status = "Rien de selectionne.".into();
            return;
        }
        match backup::save_session(backups) {
            Ok(path) => {
                self.status = format!(
                    "{count} reglage(s) applique(s). Sauvegarde : {}{}",
                    path.display(),
                    if reboot {
                        "  —  redemarrage conseille."
                    } else {
                        ""
                    }
                );
            }
            Err(e) => self.status = format!("Applique, mais sauvegarde non ecrite : {e}"),
        }
    }

    fn restore_latest(&mut self) {
        match backup::load_latest() {
            Ok(Some(session)) => {
                let mut ok = 0;
                for tb in &session.tweaks {
                    if backup::restore(tb).is_ok() {
                        ok += 1;
                    }
                }
                self.refresh();
                self.status = format!("Restauration : {ok} reglage(s) remis a leur etat anterieur.");
            }
            Ok(None) => self.status = "Aucune sauvegarde a restaurer.".into(),
            Err(e) => self.status = format!("Lecture de sauvegarde impossible : {e}"),
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
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    for n in &selected {
                        ui.label(format!("•  {n}"));
                    }
                });
                if reboot {
                    ui.add_space(4.0);
                    ui.colored_label(ORANGE, "Un redemarrage sera conseille apres application.");
                }
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
                ui.label("— telemetrie Windows 11 sous votre controle");
            });
            if !self.elevated {
                ui.colored_label(
                    Color32::from_rgb(220, 80, 80),
                    "Droits administrateur manquants : les reglages « machine » echoueront. Relancez en administrateur.",
                );
            }
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let n = self.selected_count();
                if ui
                    .button(format!("Appliquer la selection ({n})"))
                    .clicked()
                {
                    if n > 0 {
                        self.confirm_open = true;
                    } else {
                        self.status = "Rien de selectionne.".into();
                    }
                }
                if ui.button("Tout restaurer").clicked() {
                    self.restore_latest();
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
    }
}
