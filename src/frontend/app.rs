use eframe::egui;
use egui::Color32;
use egui::{Role::Label, ViewportBuilder};
use tokio::io::join;
use std::collections::{HashSet, HashMap};

use crate::backend::protocol::{BackEnd, InitializeResponse, Pending};
use crate::backend::types::{ChatId, TopicId};
use crate::{backend, frontend};
use frontend::types::{UiChat, UiTopic};
use frontend::auth::AuthFlow;
use frontend::task::drive;

pub fn run() -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default() 
            .with_maximized(true)
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native("EmbedSearch", native_options, Box::new(|c| {Ok(Box::new(App::new(c)))}))?;
    Ok(())
}


pub enum Screen {
    Auth(AuthFlow),
    Main(MainScreen),
} 


pub struct App {
    screen: Screen,
    backend: BackEnd,
}








impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let backend = BackEnd::initialize();
        let screen = Screen::Auth(AuthFlow::init());
            
        Self {
            backend,
            screen,
        }
    }
}

pub fn draw_chats_panel(ui: &mut egui::Ui, app_data: &mut AppData) {
    egui::Panel::left(egui::Id::unique("LeftP"))
        .resizable(true)
        .show(ui, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(30, 30, 30))
                .corner_radius(10.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.heading("Chats");
                    ui.horizontal(|ui| {
                        if ui.button("Add chats").clicked() {
                            app_data.active_modal = UiModal::AddChats;
                        }
                        ui.button("update");
                    });
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            match app_data.task {
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
                    
                    
                    
                });
    });
}


fn draw_add_chats(ui: &mut egui::Ui, app_data: &mut AppData) {
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
                                app_data.active_modal = UiModal::None;
                                app_data.selected_chats.clear();
                                app_data.selected_topics.clear();
                            }
                        });
                    });
                    
                    ui.add_space(10.0);
                    ui.text_edit_singleline(&mut app_data.input);
                    ui.add_space(10.0);
                    
                    let mut filtered_chats: Vec<UiChat> = app_data.available_chats.iter_mut().
                        filter(|c| c.name.to_lowercase().contains(&app_data.input.to_lowercase()))
                        .map(|c| c.clone())
                        .collect();
                    
                    let chats = if app_data.input.is_empty() {
                        &app_data.available_chats
                    } else {
                        &filtered_chats
                    };
                    
                    let rows = chats.len();

                    ui.separator();

                    let row_height = 50.0;
                    
                    egui::Frame::new()
                        .show(ui, |ui| {
                            ui.set_min_height(400.0);
                            egui::ScrollArea::vertical()
                                .max_height(400.0)
                                .max_width(550.0)
                                .show_rows(ui, row_height, rows, |ui, row_range| {
                                    
                                ui.spacing_mut().item_spacing.y = 0.0;
                                
                                for row in row_range {
                                    let chat = &chats[row];
        
                                    let selected = app_data.selected_chats.contains(&chat.id);
                                    
                                    let frame = if selected {
                                        egui::Frame::new()
                                            .fill(egui::Color32::from_rgb(45, 50, 60))                                    
                                    } else {
                                        egui::Frame::new()
                                    };
        
                                    frame
                                    .show(ui, |ui| {
                                        
                                        ui.horizontal(|ui| {
                                            
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(44.0, 44.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                rect.center(),
                                                22.0,
                                                egui::Color32::DARK_GRAY,
                                            );
                                            ui.label(&chat.name);
                                            ui.with_layout(
                                                   egui::Layout::right_to_left(egui::Align::Center),
                                                   |ui| {
                                                        if let Some(topics) = &chat.topics {
                                                            ui.menu_button("Show topics", |ui| {
                                                                ui.set_min_width(220.0);
                                                                egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                                                    let mut remove_topics_entry = false;
                                                                    if let Some(sel) = app_data.selected_topics.get_mut(&chat.id) {
                                                                        for topic in topics {
                                                                            ui.horizontal( |ui| {
                                                                                ui.label(&topic.name);
                                                                                if sel.contains(&topic) {
                                                                                    if ui.button("Cancel").clicked() {
                                                                                        sel.remove(topic); 
                                                                                        if sel.is_empty() {
                                                                                            remove_topics_entry = true;
                                                                                            app_data.selected_chats.remove(&chat.id);
                                                                                        }
                                                                                    }
                                                                                } else {
                                                                                    if ui.button("Select").clicked() {
                                                                                        sel.insert(topic.clone());
                                                                                        
                                                                                    }
                                                                                }
                                                                                
                                                                            });
                                                                            
                                                                        }
                                                                    } else {
                                                                        for topic in topics {
                                                                            ui.horizontal( |ui| {
                                                                                ui.label(&topic.name);
                                                                                if ui.button("Select").clicked() {
                                                                                    let mut h_set = HashSet::new();
                                                                                    h_set.insert(topic.clone());
                                                                                    app_data.selected_topics.insert(chat.id.clone(), h_set);
                                                                                    if !app_data.selected_chats.contains(&chat.id) {
                                                                                        app_data.selected_chats.insert(chat.id.clone());
                                                                                    }
                                                                                }
                                                                            });
                                                                            
                                                                        }
                                                                    }
                                                                    if remove_topics_entry {
                                                                        app_data.selected_topics.remove(&chat.id);
                                                                    }
                                                               });
                                                           });
                                                       } else {
                                                           if selected {
                                                               if ui.button("Cancel").clicked() {
                                                                   app_data.selected_chats.remove(&chat.id);
                                                               }
                                                           } else {
                                                               if app_data.indexed_chats.contains(chat) {
                                                                   ui.label("Indexed");
                                                               } else {
                                                                   if ui.button("Select").clicked() {
                                                                       app_data.selected_chats.insert(chat.id.clone());
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
                    
                    
                    ui.separator();
                    ui.add_space(10.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Confirm").clicked() {
                            app_data.task = BackGroundTask::AddingChats(None);
                            app_data.active_modal = UiModal::None;
                        }
                    });
                    
                });
            
        
        });
}

impl eframe::App for App {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
       match &mut self.screen {
           Screen::Auth(flow) => {
               if let Some(init_response) = flow.update(&mut self.backend) {
                   let app_data = AppData::from_init(init_response);
                   self.screen = Screen::Main(MainScreen {app_data, input: String::new()});
                   return;
               }
               flow.draw(ui);
           },
           Screen::Main(screen) => {
               
           }
       }
   }
}

