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
use rusqlite::params;
use std::sync::Arc;

use rusqlite::Connection;

use crate::backend::embedding::{self, DbIndexedImage, RawIndexedImage};
use crate::backend::types::{ChatId, MessageId};

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
    pub fn load_indexed(&self) -> anyhow::Result<Vec<DbIndexedImage>> {
        let mut stmt = self.connection.prepare("SELECT msg_id, chat_id, embedding FROM images")?;

        let rows = stmt.query_map([], |row| {
            let msg_id: i32 = row.get(0)?;
            let chat_id: i64 = row.get(1)?;
            let bytes: Vec<u8> = row.get(2)?;

            let embedding: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(
                    [chunk[0],chunk[1], chunk[2], chunk[3]]
                ))
                .collect();
            
            Ok(DbIndexedImage {
                msg_id: MessageId { id: msg_id },
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
    pub fn save_indexed(&mut self, indexed: &Vec<DbIndexedImage>) -> anyhow::Result<()> {
        let trans = self.connection.transaction()?;
        
        for img in indexed {
            let bytes: &[u8] = bytemuck::cast_slice(
                img.embedding.as_ref()
            );
            let msg_id = img.msg_id.id;
            let chat_id = img.chat_id.id;
            
            trans.execute(
              "INSERT INTO images(msg_id, chat_id, embedding) values(?1, ?2, ?3)",
              params![msg_id, chat_id, bytes]
            )?;
        }
        
        trans.commit()?;
        Ok(())
    }
}