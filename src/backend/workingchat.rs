
use anyhow::anyhow;
use grammers_client::{Client, media::Media};
use grammers_session::Session;
use grammers_session::storages::SqliteSession;
use grammers_mtsender::SenderPool;
use grammers_session::types::PeerId;
use image::{DynamicImage, GenericImageView};
use ndarray::{Array4, Array2};
use ort::environment::Environment;
use ort::value::{Tensor, Value};
use image::ImageReader;
use std::path::Path;
use std::sync::Arc;

use grammers_session::types::PeerRef;
use grammers_client::peer::Peer;
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


