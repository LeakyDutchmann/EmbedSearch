use super::MainScreen;
use super::{UiModal, BackGroundTask, AppData};

impl MainScreen {
    pub fn draw_chats_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left(egui::Id::unique("LeftP"))
            .resizable(true)
            .show(ui, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(30, 30, 30))
                    .corner_radius(10.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {     
                        draw_header(ui, &mut self.modal);
                        draw_chats_list(ui, &self.task, &self.app_data);
                    });
        });
    }
}

fn draw_chats_list(ui: &mut egui::Ui, task: &BackGroundTask, app_data: &AppData) {
    egui::ScrollArea::vertical()
        .show(ui, |ui| {
            match task {
                BackGroundTask::AddingChats(_) => {
                    egui::Frame::new()
                        .show(ui, |ui| {
                            ui.label("Setting working chats...");
                            ui.add_space(10.0);
                            ui.spinner();
                        });
                }
                BackGroundTask::Refreshing(_) => {
                    egui::Frame::new()
                        .show(ui, |ui| {
                            ui.label("Refreshing data...");
                            ui.add_space(10.0);
                            ui.spinner();
                        });
                },
                BackGroundTask::Indexing(_) => {
                    egui::Frame::new()
                        .show(ui, |ui| {
                            ui.label("Indexing images...");
                            ui.add_space(10.0);
                            ui.spinner();
                        });
                },
                _ => {
                    for chat in &app_data.indexed_chats {
                        egui::Frame::new()
                            .show(ui, |ui| {
                                egui::Frame::new()
                                    .show(ui, |ui| {
                                        ui.label(&chat.name);
                                        if let Some(topics) = &chat.topics {
                                            ui.separator();
                                            for topic in topics {
                                                ui.label(&topic.name);
                                            }
                                        }
                                    });
                                
                            });
                    }
                }
            }
        });
}

fn draw_header(ui: &mut egui::Ui, modal: &mut UiModal) {
    ui.heading("Chats");
    ui.horizontal(|ui| {
        if ui.button("Add chats").clicked() {
            *modal = UiModal::AddChats;
        }
        ui.button("update");
    });
    ui.separator();
}