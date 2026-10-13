//! Interface graphique (egui / eframe). Branche sur le meme coeur que la logique
//! metier : catalogue, detection, application et restauration restent inchanges.

use crate::model::{Category, Tweak, TweakState};
use crate::{backup, registry, tweaks};
use eframe::egui;
use egui::Color32;

struct Row {
    tweak: Tweak,
    state: TweakState,
    selected: bool,
}

pub struct QuietusApp {
    rows: Vec<Row>,
    focused: usize,
    filter: Option<Category>,
    status: String,
    elevated: bool,
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
            filter: None,
            status: "Cochez des reglages, puis « Appliquer la selection ».".into(),
            elevated: registry::is_elevated(),
        }
    }

    fn refresh(&mut self) {
        for r in &mut self.rows {
            r.state = tweaks::detect(&r.tweak);
        }
        self.status = "Etats actualises.".into();
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
}

fn state_badge(ui: &mut egui::Ui, s: TweakState) {
    let (txt, col) = match s {
        TweakState::Disabled => ("desactive", Color32::from_rgb(80, 180, 80)),
        TweakState::Enabled => ("actif", Color32::from_rgb(210, 170, 60)),
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
                if ui.button("Appliquer la selection").clicked() {
                    self.apply_selected();
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

        egui::SidePanel::left("cats").resizable(false).show(ctx, |ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Categories").strong());
            ui.add_space(4.0);
            if ui.selectable_label(self.filter.is_none(), "Toutes").clicked() {
                self.filter = None;
            }
            for c in Category::ALL {
                if ui.selectable_label(self.filter == Some(c), c.label()).clicked() {
                    self.filter = Some(c);
                }
            }
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
                        ui.colored_label(
                            Color32::from_rgb(210, 170, 60),
                            "Redemarrage requis apres application.",
                        );
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
                for i in 0..self.rows.len() {
                    let visible = match self.filter {
                        None => true,
                        Some(c) => self.rows[i].tweak.category == c,
                    };
                    if !visible {
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
            });
        });
    }
}
