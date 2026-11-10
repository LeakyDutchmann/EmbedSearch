use crate::backend::{self, models::Embedder, telegram::TelegramClient, types::WorkingChat};

pub struct SearchEngine {
    embedder: Embedder,
    telegram_client: TelegramClient,
    working_chats: Vec<WorkingChat>
}

impl SearchEngine {
    pub async fn initialize() -> anyhow::Result<Self>{
        dotenvy::dotenv().ok(); 
        let api_id: i32 = std::env::var("TELEGRAM_API_ID")?.parse()?;
        
        let embedder = backend::models::Embedder::initialize("models/model.onnx")?;
        let telegram_client = TelegramClient::intialize("session/session.db", api_id).await?;

        Ok(SearchEngine {
            embedder,
            telegram_client,
            working_chats: Vec::new(),
        })
    }
}