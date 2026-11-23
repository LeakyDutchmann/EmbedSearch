use super::{AuthFlow, AuthState};
use crate::frontend::widgets::show_centered;
use egui::Color32;


impl AuthFlow {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        match &mut self.state {
            AuthState::Checking(_) => { 
                show_centered(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Fetching session!");
                        ui.add_space(10.0);
                        ui.spinner();
                    });
                });
            }
            AuthState::EnterPhone => { 
                &mut self.draw_enter_phone(ui);
                
            }
            AuthState::RequestingCode(_) => { 
                &mut self.draw_requesting_code(ui);
            }
            AuthState::EnterCode => { 
                &mut self.draw_enter_code(ui);
            }
            AuthState::SigningIn(_) => {
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
            AuthState::Error(e) => {
                show_centered(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let line = format!("Error: {}", e);
                        ui.colored_label(Color32::LIGHT_RED, line);
                    });
                });
            }
        }
    }
    fn draw_enter_phone(&mut self, ui: &mut egui::Ui) {
        show_centered(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Paste you phone number to proceed the authorization");
                ui.add_space(10.0);
                ui.add_sized(
                    [300.0, 24.0],
                    egui::TextEdit::singleline(&mut self.input),
                );
                if ui.button("Submit").clicked() {
                    let num = self.input.trim().to_string();
                    println!("{}", num.clone());
                    self.input.clear();
                    self.phone = Some(num);
                    self.state = AuthState::RequestingCode(None)
                }
            });
        });
    }
    fn draw_requesting_code(&mut self, ui: &mut egui::Ui) {
        show_centered(ui, |ui| {
            ui.vertical_centered(|ui| {
                let line = format!("Code was sent to {}", self.phone.as_ref().unwrap());
                ui.label(line);
                ui.add_space(10.0);
                ui.text_edit_singleline(&mut self.input);
                if ui.button("Submit").clicked() {
                    self.phone = Some(self.input.clone());
                    self.state = AuthState::RequestingCode(None)
                }
                ui.add_space(10.0);
                if ui.button("Cancel").clicked() {
                    self.state = AuthState::EnterPhone
                }
            });
        }); 
    }
    fn draw_enter_code(&mut self, ui: &mut egui::Ui) {
        show_centered(ui, |ui| {
            ui.vertical_centered(|ui| {
                let line = format!("Code was sent to {}", self.phone.as_ref().unwrap());
                ui.label(line);
                ui.add_space(10.0);
                ui.text_edit_singleline(&mut self.input);
                if ui.button("Submit").clicked() {
                    let num = self.input.trim().to_string();
                    self.code = Some(num);
                    self.input.clear();
                    self.state = AuthState::SigningIn(None)
                }
                if ui.button("Cancel").clicked() {
                    self.state = AuthState::EnterPhone
                }
            });
        });
    }
}