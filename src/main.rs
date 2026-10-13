//! Quietus : gestionnaire de telemetrie Windows 11, controle par l'utilisateur.
//! Interface graphique (egui). Tout reglage applique est sauvegarde et reversible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod gui;
mod model;
mod registry;
mod tasks;
mod tweaks;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([920.0, 600.0])
            .with_min_inner_size([760.0, 460.0])
            .with_title("Quietus"),
        ..Default::default()
    };
    eframe::run_native(
        "Quietus",
        options,
        Box::new(|_cc| Ok(Box::new(gui::QuietusApp::new()))),
    )
}
