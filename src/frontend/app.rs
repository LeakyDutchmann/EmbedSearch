use eframe::egui;
use egui::{Role::Label, ViewportBuilder};
use tokio::io::join;

use crate::backend::protocol::{Pending, BackEnd};

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
    Authorized,
}


pub struct App {
    backend: BackEnd,

    state: AppState,
    
    phone_input: String,

    phone: Option<String>,
    code: Option<String>,

}

struct AppData {
    
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
                                        *a_state = AuthState::Authorized;
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
                                    *a_state = AuthState::Authorized
                                }
                            }
                        }
                    }
                    AuthState::Authorized => { 
                       self.state = AppState::Authorized(AppData {  })
                    }
                }
            },
            AppState::Authorized(app_data) => {
                
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
                                ui.spinner();
                            });
                        });
                    }
                    AuthState::Authorized => { 
                        show_centered(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let line = format!("Welcome!...");
                                ui.label(line);
                                
                            });
                        });
                    }
                }
            },
            AppState::Authorized(app_data) => {
                egui::Panel::left(egui::Id::unique("LeftP"))
                    .resizable(true)
                    .show(ui, |ui| {
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(30, 30, 30))
                            .corner_radius(10.0)
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.heading("Chats");
                                ui.separator();
                                
                                egui::ScrollArea::vertical()
                                    .show(ui, |ui| {
                                        for _ in 0..70 {
                                            ui.add_sized(
                                                [ui.available_width() - 10.0, 0.0],
                                                egui::Button::new("chatx"),
                                    );
                                    }
                        });
                    });
                });
            }
        }
        
    }
}

fn show_centered<F>(ui: &mut egui::Ui, f: F) where F: FnOnce(&mut egui::Ui) {
    egui::Area::new(egui::Id::unique("login"))
     .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
     .show(ui, |ui| {
         f(ui)
     });
}

impl eframe::App for App {
   fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
      self.update_state();
      self.draw_ui(ui);
   }
}

