
use grammers_client::{Client, media::Media};
use grammers_session::Session;
use grammers_session::storages::SqliteSession;
use grammers_mtsender::SenderPool;
use image::{DynamicImage, GenericImageView};
use ndarray::{Array4, Array2};
use ort::environment::Environment;
use ort::value::{Tensor, Value};
use image::ImageReader;
use std::path::Path;
use std::sync::Arc;
use grammers_client::client::LoginToken;
use grammers_client::peer::User;

use crate::backend::types::WorkingChat;


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
    pub async fn get_available_chats(&self) -> anyhow::Result<Vec<WorkingChat>> {
        let mut chats: Vec<WorkingChat> = Vec::new();

        let mut dialogs = self.client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            let peer = dialog.peer();
            chats.push(WorkingChat {
                name: peer.name().unwrap_or("Unknown").to_string()
            });
        }
        Ok(chats)
    }
}