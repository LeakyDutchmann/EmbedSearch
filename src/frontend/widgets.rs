
pub fn show_centered<F>(ui: &mut egui::Ui, f: F) where F: FnOnce(&mut egui::Ui) {
    egui::Area::new(egui::Id::unique("login"))
     .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
     .show(ui, |ui| {
         f(ui)
     });
}

pub fn draw_avatar_placeholder(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(44.0, 44.0),
        egui::Sense::hover(),
    );
    ui.painter().circle_filled(
        rect.center(),
        22.0,
        egui::Color32::DARK_GRAY,
    );
}