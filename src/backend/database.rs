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

use rusqlite::Connection;

use crate::backend::embedding::IndexedImage;
use crate::backend::workingchat::ChatId;

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn initialize(database_path: impl AsRef<Path> ) -> anyhow::Result<Self> {
        let connection = Connection::open(database_path)?;

        connection.execute("CREATE TABLE IF NOT EXISTS images (
            embedding BLOB NOT NULL,
            msg_id INT NOT NULL,
            chat_id INT NOT NULL,
            
            PRIMARY KEY (chat_id, msg_id)
        );", ())?;
        
        Ok(Database {
            connection
        })
    }
    pub fn load_indexed(&self) -> anyhow::Result<Vec<IndexedImage>> {
        let mut stmt = self.connection.prepare("SELECT msg_id, chat_id, embedding FROM images")?;

        let rows = stmt.query_map([], |row| {
            let msg_id = row.get(0)?;
            let chat_id: i64 = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;

            let embedding: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(
                    [chunk[0],chunk[1], chunk[2], chunk[3]]
                ))
                .collect();
            
            Ok(IndexedImage {
                msg_id,
                chat_id: ChatId{ id: chat_id},
                embedding: embedding.into_boxed_slice()
            })
        })?;
        let mut result = Vec::new();

        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }
}