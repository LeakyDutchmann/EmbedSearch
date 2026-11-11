
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
use crate::backend::embedding::IndexedImage;

pub struct Chat {
    pub chat_id: ChatId,
    pub name: String,
    pub peer_ref: PeerRef,
    pub indexed_images: Vec<IndexedImage>
}

pub struct WorkingChat {
    pub name: String,
    pub peer_ref: PeerRef,
    pub indexed_images: Vec<IndexedImage>
}

impl WorkingChat {
    pub fn from_chat(chat: Chat) -> Self {
        WorkingChat {
            name: chat.name,
            peer_ref: chat.peer_ref,
            indexed_images: chat.indexed_images
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct ChatId {
    pub id: i64
}

impl ChatId {
    pub fn from_peer_ref(peer_ref: PeerRef) -> anyhow::Result<Self> {
        if let Some(id) = peer_ref.id.bare_id() {
            Ok(ChatId {id})
        } else {
            Err(anyhow!("Failed to get peer id"))
        }
        
    }
}