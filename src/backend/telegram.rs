
use anyhow::anyhow;
use crate::backend;
use grammers_client::{Client, media::Media};
use grammers_session::storages::SqliteSession;
use grammers_mtsender::SenderPool;
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;
use grammers_client::client::LoginToken;
use grammers_session::types::PeerRef;
use backend::workingchat::{Chat, WorkingChat};
use std::collections::HashMap;
use backend::types::{ChatId, PendingImage, MessageId};



pub struct TelegramClient {
    pub client: Client
}


impl TelegramClient {
    pub async fn intialize(session_path: impl AsRef<Path>, telegram_api_id: i32) -> anyhow::Result<Self> {
        let session = Arc::new(SqliteSession::open(session_path).await?);
        let pool = SenderPool::new(Arc::clone(&session), telegram_api_id);
        let client = Client::new(pool.handle);

        tokio::spawn(async move {
            let result = pool.runner.run().await;
            println!("Telegram's SenderPoolRunner finished: {:?}", result);
        });

        Ok(TelegramClient {
            client
        })
    }
    pub async fn request_login_code(&self, api_hash: String, phone: String) -> anyhow::Result<LoginToken> {
        let token = self.client.request_login_code(&phone, &api_hash).await?;
        Ok(token)
    }
    pub async fn sign_in(&self, code: &str, token: &LoginToken) -> anyhow::Result<()> {
        let _user = self.client.sign_in(token, code).await?;
        Ok(())
    }
    pub async fn is_authorized(&self) -> anyhow::Result<bool> {
        let bool = self.client.is_authorized().await?;
        Ok(bool)
    }
    pub async fn get_available_chats(&self) -> anyhow::Result<Vec<Chat>> {
        let mut chats: Vec<Chat> = Vec::new();

        let mut dialogs = self.client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            
            let peer = dialog.peer();
            let peer_ref = match peer.to_ref().await {
                Ok(Some(p_r)) => p_r,
                _ => {return Err(anyhow!("Failed to get peer reference"))}
            };
            
            chats.push(Chat {
                chat_id: ChatId::from_peer_ref(peer_ref.clone())?,
                peer_ref,
                name: peer.name().unwrap_or("Unknown").to_string(),
            });
            
        }
        Ok(chats)
    }
    pub async fn get_unindexed_images(&self, chat: &WorkingChat) -> anyhow::Result<Vec<PendingImage>> {
        let mut messages = self.client.iter_messages(chat.peer_ref);

        
        let mut images = Vec::new();
        let mut bytes = Vec::new();
        
        while let Some(msg) = messages.next().await? {
            let Some(media) = msg.media() else {
                continue 
            };

            let msg_id = MessageId{id: msg.id()};
            
            if chat.indexed_images.contains_key(&msg_id) {
                continue;
            }
            
            match media {
                Media::Photo(photo) => {
                    bytes.clear();
                    
                    let mut download = self.client.iter_download(&photo);
        
                    while let Some(chunk) = download.next().await? {
                        bytes.extend(chunk);
                    }
        
                    let img = image::load_from_memory(&bytes)?;
                    images.push(PendingImage {
                        img,
                        msg_id
                    });             
                },
                _ => continue
            }
        }
        
        Ok(images)
    }
    pub async fn download_images(&self, peer_ref: PeerRef, msg_ids: &[MessageId] ) -> anyhow::Result<HashMap<MessageId, DynamicImage>> {
        let message_ids: Vec<i32> = msg_ids.iter().map(|s| s.id).collect();
        let messages = self.client.get_messages_by_id(peer_ref, &message_ids).await?;

        let messages: Vec<grammers_client::message::Message> = messages.into_iter().filter(|m| m.is_some()).map(|m| m.unwrap()).collect();
        let count = message_ids.len() - messages.len();
        println!("Lost {} messages", count);

        let mut images: HashMap<MessageId, DynamicImage> = HashMap::new();
        
        let mut bytes: Vec<u8> = Vec::new();
        for msg in messages {
            let Some(media) = msg.media() else {
                continue;
            };

            let id = msg.id();
            
            bytes.clear();
            match media {
                Media::Photo(photo) => {
                    let mut download = self.client.iter_download(&photo);

                    while let Some(chunk) = download.next().await? {
                        bytes.extend(chunk);
                    }
                    
                    let Ok(img) = image::load_from_memory(&bytes) else {
                        continue;
                    };
                    
                    images.insert(MessageId { id }, img);
                },
                _ => {
                    continue;
                }
            }
        }
        Ok(images)
    }
}