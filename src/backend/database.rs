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

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn initialize(database_path: impl AsRef<Path> ) -> anyhow::Result<Self> {
        let connection = Connection::open(database_path)?;

        connection.execute("CREATE TABLE IF NOT EXISTS images (
            embedding BLOB NOT NULL,
            msg_id INT PRIMARY KEY,
            chat_id INT NOT NULL
        );", ())?;
        
        Ok(Database {
            connection
        })
    }
}