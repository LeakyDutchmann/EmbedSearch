
use super::MainScreen;
use super::Selection;
use super::{UiModal, BackGroundTask, AppData};
use super::types::{UiTopic, UiChat};

struct RowData {
    height: f32,
    count: usize,
}

impl MainScreen {
    pub fn draw_add_chats(&mut self, ui: &mut egui::Ui) {
        egui::Modal::new(egui::Id::unique("AddChats"))
            .show(ui, |ui| {
                ui.set_min_width(500.0);
                ui.set_min_height(500.0);
                egui::Frame::new()
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading("Select chats for indexing");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Cancel").clicked() {
                                    self.modal = UiModal::None;
                                    self.selection.clear();
                                }
                            });
                        });
                        
                        ui.add_space(10.0);
                        ui.text_edit_singleline(&mut self.input);
                        ui.add_space(10.0);
                        
                        let filtered_chats: Vec<UiChat> = self.app_data.available_chats.iter_mut().
                            filter(|c| c.name.to_lowercase().contains(&self.input.to_lowercase()))
                            .map(|c| c.clone())
                            .collect();
                        
                        let chats = if self.input.is_empty() {
                            &self.app_data.available_chats.clone()
                        } else {
                            &filtered_chats
                        };
                        
                        ui.separator();
                        
                        let rows = chats.len();
                        let row_height = 50.0;
                        
                        let row = RowData {
                            height: row_height,
                            count: rows,
                        };
                        
                        draw_adder_scrollarea(
                            ui,
                            row,
                            chats,
                            &mut self.selection,
                            &self.app_data
                        );
                        
                        ui.separator();
                        ui.add_space(10.0);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Confirm").clicked() {
                                self.task = BackGroundTask::AddingChats(None);
                                self.modal = UiModal::None;
                            }
                        });
                        
                    });
                
            
            });
    }
}

fn draw_adder_scrollarea(
    ui: &mut egui::Ui,
    row: RowData,
    chats: &Vec<UiChat>,
    selection: &mut Selection,
    app_data: &AppData,
) {
    egui::Frame::new()
        .show(ui, |ui| {
            ui.set_min_height(400.0);
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .max_width(550.0)
                .show_rows(ui, row.height, row.count, |ui, row_range| {
                    
                ui.spacing_mut().item_spacing.y = 0.0;
                
                for row in row_range {
                    let chat = &chats[row];

                    let selected = selection.is_chat_selected(&chat.id);
                    
                    let frame = if selected {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(45, 50, 60))                                    
                    } else {
                        egui::Frame::new()
                    };

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| { 
                            
                            super::draw_avatar_placeholder(ui);

                            ui.label(&chat.name);
                            ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if let Some(topics) = &chat.topics {
                                            ui.menu_button("Show topics", |ui| {
                                                ui.set_min_width(220.0);
                                                draw_topics_scrollarea(ui, topics, chat, selection);
                                            });
                                        } else {
                                            if selected {
                                                if ui.button("Cancel").clicked() {
                                                    selection.remove_chat(&chat.id);
                                                }
                                            } else {
                                                if app_data.is_chat_indexed(chat) {
                                                    ui.label("Indexed");
                                                } else {
                                                    if ui.button("Select").clicked() {
                                                        selection.select_chat(&chat.id);
                                                    } 
                                                }
                                            }
                                        }
                                        
                                        
                                },
                            );
                            
                        });
                    });
                    
                }
            });
        });
}

fn draw_topics_scrollarea(
    ui: &mut egui::Ui,
    topics: &Vec<UiTopic>,
    chat: &UiChat,
    selection: &mut Selection,
) {
    egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
        for topic in topics {
            ui.horizontal( |ui| {
                ui.label(&topic.name);
                let on = selection.is_topic_selected(&chat.id, &topic.id);
                if ui.button(if on {"Cancel"} else {"Select"}).clicked() {
                    selection.toggle_topic(&chat.id, topic);
                }
            }); 
        }
   });
}