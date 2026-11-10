
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
}