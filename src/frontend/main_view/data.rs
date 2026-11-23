use std::collections::HashSet;
use super::InitializeResponse;

use super::types::UiChat;

pub struct AppData {
    indexed_chats: HashSet<UiChat>,
    available_chats: Vec<UiChat>,
}

impl AppData {
    pub fn from_init(res: InitializeResponse) -> Self {
        let indexed_chats: HashSet<UiChat> = res.indexed_chats.into_iter().map(|c| UiChat::from_info(c)).collect();
        let available_chats: Vec<UiChat> = res.available_chats.into_iter().map(|c| UiChat::from_info(c)).collect();
        AppData {
            indexed_chats,
            available_chats,
        }
    }
}