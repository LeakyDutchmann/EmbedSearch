
pub fn show_centered<F>(ui: &mut egui::Ui, f: F) where F: FnOnce(&mut egui::Ui) {
    egui::Area::new(egui::Id::unique("login"))
     .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
     .show(ui, |ui| {
         f(ui)
     });
}
