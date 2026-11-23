pub mod chats_panel;
pub mod add_chats;
pub mod data;
pub mod selection;
pub mod types;
pub mod enums;

use crate::backend::types::{ChatId, TopicId};
use crate::backend::protocol::{Pending, InitializeResponse, BackEnd};
use super::task::drive;

use selection::Selection;
use data::AppData;
use enums::{UiModal, BackGroundTask};


pub struct MainScreen {
    app_data: AppData,
    selection: Selection,
    modal: UiModal,
    error: Option<String>,
    task: BackGroundTask,
    input: String,
}

impl MainScreen {
    fn step(&mut self, backend: &BackEnd) -> anyhow::Result<()> {
        match &mut self.task {
            BackGroundTask::AddingChats(p) => {
                if let Some(res) = drive(p, || backend.set_working_chats(self.selection.get_selected_chats())) {
                    res?;
                    self.selection.clear();
                    self.task = BackGroundTask::Refreshing(None);
                }
            },
            BackGroundTask::Refreshing(p) => {
                if let Some(init_r) = drive(p, || backend.get_app_data()) {
                    let data = AppData::from_init(init_r?);
                    self.app_data = data;
                    self.task = BackGroundTask::Indexing(None);
                }
            }
            BackGroundTask::Indexing(p) => {
                if let Some(res) = drive(p, || backend.index_chats()) {
                    res?;
                    self.task = BackGroundTask::None;
                }
            },
            BackGroundTask::None => {
                
            }
        }
        Ok(())
    }
    pub fn update(&mut self, backend: &BackEnd) {
        match self.step(backend) {
            Ok(_) => {
                
            },
            Err(e) => {
                println!("Error ocured: {}", e);
                self.task = BackGroundTask::None;
            }
        }
    }
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        if self.modal == UiModal::AddChats {
            draw_add_chats(ui, &mut self.app_data);
        }
        draw_chats_panel(ui, &mut self.app_data);
    }
}