use grammers_session::types::PeerRef;
use anyhow::anyhow;
use image::DynamicImage;

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct ChatId {
    pub id: i64
}


#[derive(Eq, Hash, PartialEq, Clone)]
pub struct MessageId {
    pub id: i32
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

pub struct PendingImage {
    pub img: DynamicImage,
    pub msg_id: MessageId,
}