use grammers_session::types::PeerRef;
use std::collections::HashMap;
use crate::backend::embedding::IndexedImage;
use crate::backend::types::{ChatId, MessageId};

pub struct Chat {
    pub chat_id: ChatId,
    pub name: String,
    pub peer_ref: PeerRef,
}


pub struct WorkingChat {
    pub name: String,
    pub peer_ref: PeerRef,
    pub indexed_images: HashMap<MessageId, IndexedImage>
}

impl WorkingChat {
    pub fn from_chat(chat: Chat) -> Self {
        WorkingChat {
            name: chat.name,
            peer_ref: chat.peer_ref,
            indexed_images: HashMap::new()
        }
    }
}


