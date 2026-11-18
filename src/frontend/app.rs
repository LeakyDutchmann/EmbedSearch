use eframe::egui;
use egui::Color32;
use egui::{Role::Label, ViewportBuilder};
use tokio::io::join;
use std::collections::HashSet;

use crate::backend::protocol::{BackEnd, InitializeResponse, Pending};
use crate::backend::types::ChatId;
use crate::frontend;
use frontend::types::UiChat;

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


enum AuthState {
    Checking(Option<Pending<bool>>),
    EnterPhone,
    RequestingCode(Option<Pending<()>>),
    EnterCode,
    SigningIn(Option<Pending<()>>),
    Authorized(Option<Pending<InitializeResponse>>),
}


pub struct App {
    backend: BackEnd,

    state: AppState,
    
    phone_input: String,

    phone: Option<String>,
    code: Option<String>,

}


#[derive(PartialEq)]
enum UiModal {
    AddChats,
    None,
}

enum BackGroundTask {
    AddingChats(Option<Pending<()>>),
    Refreshing(Option<Pending<InitializeResponse>>),
    Indexing(Option<Pending<()>>),
    None,
}

struct AppData {
    indexed_chats: HashSet<UiChat>,
    available_chats: Vec<UiChat>,
    selected_chats: HashSet<ChatId>,
    active_modal: UiModal,
    input: String,
    task: BackGroundTask,
}

impl AppData {
    fn from_init(res: InitializeResponse) -> Self {
        let indexed_chats: HashSet<UiChat> = res.indexed_chats.iter().map(|c| UiChat {name: c.name.clone(), id: c.chat_id.clone()}).collect();
        let available_chats: Vec<UiChat> = res.available_chats.iter().map(|c| UiChat { name: c.name.clone(), id: c.chat_id.clone()}).collect();
        AppData {
            indexed_chats,
            available_chats,
            selected_chats: HashSet::new(),
            task: BackGroundTask::None,
            active_modal: UiModal::None,
            input: String::new()
        }
    }
}

enum AppState {
    Authentication(AuthState),
    Authorized(AppData)
}


impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let backend = BackEnd::initialize();
            
        Self {
            backend,
            state: AppState::Authentication(AuthState::Checking(None)),
            phone_input: String::new(),
            phone: None,
            code: None,
        }
    }
    fn update_state(&mut self) {
        match &mut self.state {
            AppState::Authentication(a_state) => {
                match a_state {
                    AuthState::Checking(p) => { 
                        if p.is_none() {
                            let pending = self.backend.is_authorized();
                            if pending.is_ok() {
                                *p = Some(pending.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(authorized) = v.try_take() {
                                    if authorized {
                                        *a_state = AuthState::Authorized(None);
                                    } else {
                                        *a_state = AuthState::EnterPhone;
                                    }
                                }
                            }
                        }
                    }
                    AuthState::EnterPhone => { 
                        
                    }
                    AuthState::RequestingCode(p) => { 
                        if p.is_none() {
                            let phone = self.phone.as_ref().unwrap().clone();
                            let pending = self.backend.request_login_code(phone);
                            if pending.is_ok() {
                                *p = Some(pending.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(res) = v.try_take() {
                                    *a_state = AuthState::EnterCode;
                                }
                            }
                            
                        }
                    }
                    AuthState::EnterCode => { 
                        
                    }
                    AuthState::SigningIn(p) => { 
                        if p.is_none() {
                            let pending = self.backend.sign_in(
                                self.code.as_ref().unwrap().clone()
                            );
                            if pending.is_ok() {
                                *p = Some(pending.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(res) = v.try_take() {
                                    *a_state = AuthState::Authorized(None)
                                }
                            }
                        }
                    }
                    AuthState::Authorized(p_data) => { 
                        if p_data.is_none() {
                            let pending = self.backend.restore_environment();
                            if pending.is_ok() {
                                *p_data = Some(pending.unwrap());
                            }
                        } else {
                            if let Some(v) = p_data {
                                if let Some(res) = v.try_take() {
                                    let app_data = AppData::from_init(res);
                                    println!("Indexed chats: {}", app_data.indexed_chats.len());
                                    self.state = AppState::Authorized(app_data)
                                }
                            }
                        }
                        
                        
                        
                    }
                }
            },
            AppState::Authorized(app_data) => {
                match &mut app_data.task {
                    BackGroundTask::AddingChats(p) => {
                        if p.is_none() {
                            let pd = self.backend.set_working_chats(&app_data.selected_chats);
                            if pd.is_ok() {
                                app_data.selected_chats.clear();
                                *p = Some(pd.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(_res) = v.try_take() {
                                    app_data.task = BackGroundTask::Refreshing(None)
                                }
                            }
                        }
                    },
                    BackGroundTask::Refreshing(p) => {
                        if p.is_none() {
                            let pd = self.backend.get_app_data();
                            if pd.is_ok() {
                                *p = Some(pd.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(res) = v.try_take() {
                                    let data = AppData::from_init(res);
                                    *app_data = data;
                                    app_data.task = BackGroundTask::Indexing(None)
                                }
                            }
                        }
                    }
                    BackGroundTask::Indexing(p) => {
                        if p.is_none() {
                            let pd = self.backend.index_chats();
                            if pd.is_ok() {
                                *p = Some(pd.unwrap());
                            }
                        } else {
                            if let Some(v) = p {
                                if let Some(_res) = v.try_take() {
                                    app_data.task = BackGroundTask::None
                                }
                            }
                        }
                    },
                    BackGroundTask::None => {
                        
                    }
                }
            }
            
        }
    }
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        match &mut self.state {
            AppState::Authentication(a_state) => {
                match a_state {
                    AuthState::Checking(p) => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label("Fetching session!");
                                ui.add_space(10.0);
                                ui.spinner();
                            });
                        });
                    }
                    AuthState::EnterPhone => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label("Paste you phone number to proceed the authorization");
                                ui.add_space(10.0);
                                ui.add_sized(
                                    [300.0, 24.0],
                                    egui::TextEdit::singleline(&mut self.phone_input),
                                );
                                if ui.button("Submit").clicked() {
                                    let num = self.phone_input.trim().to_string();
                                    println!("{}", num.clone());
                                    self.phone_input.clear();
                                    self.phone = Some(num);
                                    *a_state = AuthState::RequestingCode(None)
                                }
                            });
                        });
                    }
                    AuthState::RequestingCode(p) => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let line = format!("Code was sent to {}", self.phone.as_ref().unwrap());
                                ui.label(line);
                                ui.add_space(10.0);
                                ui.text_edit_singleline(&mut self.phone_input);
                                if ui.button("Submit").clicked() {
                                    self.phone = Some(self.phone_input.clone());
                                    *a_state = AuthState::RequestingCode(None)
                                }
                                ui.add_space(10.0);
                                if ui.button("Cancel").clicked() {
                                    *a_state = AuthState::EnterPhone
                                }
                            });
                        }); 
                    }
                    AuthState::EnterCode => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let line = format!("Code was sent to {}", self.phone.as_ref().unwrap());
                                ui.label(line);
                                ui.add_space(10.0);
                                ui.text_edit_singleline(&mut self.phone_input);
                                if ui.button("Submit").clicked() {
                                    let num = self.phone_input.trim().to_string();
                                    self.code = Some(num);
                                    self.phone_input.clear();
                                    *a_state = AuthState::SigningIn(None)
                                }
                                if ui.button("Cancel").clicked() {
                                    *a_state = AuthState::EnterPhone
                                }
                            });
                        });
                    }
                    AuthState::SigningIn(p) => {
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let line = format!("Signing in...");
                                ui.label(line);
                                ui.add_space(10.0);
                                ui.spinner();
                            });
                        });
                    }
                    AuthState::Authorized(_) => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let line = format!("Fetching data!...");
                                ui.label(line);
                                ui.add_space(10.0);
                                ui.spinner();
                                
                            });
                        });
                    }
                }
            },
            AppState::Authorized(app_data) => {
                if app_data.active_modal == UiModal::AddChats {
                    draw_add_chats(ui, app_data);
                }
                draw_chats_panel(ui, app_data);
            }
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
                                                ui.label(&chat.name)
                                            });
                                    }
                                }
                            }
                        });
                    
                    
                    
                });
    });
}

fn show_centered<F>(ui: &mut egui::Ui, f: F) where F: FnOnce(&mut egui::Ui) {
    egui::Area::new(egui::Id::unique("login"))
     .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
     .show(ui, |ui| {
         f(ui)
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
      self.update_state();
      self.draw_ui(ui);
   }
}

