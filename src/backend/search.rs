use crate::backend::{self, database::Database, embedding::{Embedder, IndexedImage, DbIndexedImage}, telegram::TelegramClient, types::ChatId, workingchat::{Chat, WorkingChat}};

use grammers_client::client::LoginToken;
use grammers_client::peer::User;
use std::collections::HashMap;

pub struct SearchEngine {
    embedder: Embedder,
    telegram_client: TelegramClient,
    database: Database,
    working_chats: HashMap<ChatId, WorkingChat>,
    login_token: Option<LoginToken>,
}

impl SearchEngine {
    pub async fn initialize() -> anyhow::Result<Self>{
        dotenvy::dotenv().ok(); 
        let api_id: i32 = std::env::var("TELEGRAM_API_ID")?.parse()?;
        
        
        let embedder = backend::embedding::Embedder::initialize("models/model.onnx")?;
        let telegram_client = TelegramClient::intialize("session/session.db", api_id).await?;
        let database = Database::initialize("data/database.db")?;

        Ok(SearchEngine {
            embedder,
            telegram_client,
            working_chats: HashMap::new(),
            database,
            login_token: None,
        })
    }
    pub async fn request_login_code(&mut self, phone: String) -> anyhow::Result<()> {
        dotenvy::dotenv().ok(); 
        let api_hash = std::env::var("TELEGRAM_API_HASH")?;

        let token = self.telegram_client.request_login_code(api_hash, phone).await?;
        self.login_token = Some(token);
        Ok(())
    }
    pub async fn sign_in(&mut self, code: &str) -> anyhow::Result<()> {
        let token = &self.login_token.take().ok_or(anyhow::anyhow!("No login token found"))?;
        self.telegram_client.sign_in(code, token).await?;
        Ok(())
    }
    pub async fn is_authorized(&self) -> anyhow::Result<bool> {
        let bool = self.telegram_client.is_authorized().await?;
        Ok(bool)
    }

    pub async fn get_available_chats(&self) -> anyhow::Result<Vec<Chat>> {
        self.telegram_client.get_available_chats().await
    }
    pub fn set_working_chats(&mut self, mut chats: Vec<Chat>) {
        while let Some(chat) = chats.pop() {
            self.working_chats.insert(chat.chat_id.clone(), WorkingChat::from_chat(chat));
        }
    }
    pub async fn load_indexed(&mut self) -> anyhow::Result<()> {
        let indexed = self.database.load_indexed()?;

        for img in indexed {
            if let Some(chat) = self.working_chats.get_mut(&img.chat_id) {
                chat.indexed_images.insert(img.msg_id, IndexedImage {embedding: img.embedding});
            }
        }
        
        Ok(())
    }
    pub async fn index_chats(&mut self) -> anyhow::Result<()> {
        for (chat_id, chat) in &mut self.working_chats {
            let images = self.telegram_client.get_unindexed_images(chat).await?;

            let raw_indexed = self.embedder.index_images(images)?;
            let mut indexed = Vec::new();

            for raw in raw_indexed {
                indexed.push( DbIndexedImage {
                    chat_id: chat_id.clone(),
                    msg_id: raw.msg_id,
                    embedding: raw.embedding,
                });
            }

            self.database.save_indexed(&indexed)?;

            for img in indexed {
                chat.indexed_images.insert(img.msg_id.clone(), IndexedImage { embedding: img.embedding });
            }
        }
        Ok(())
    }
}