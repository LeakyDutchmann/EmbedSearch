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

pub struct Embedder {
    pub session: ort::session::Session, 
}           

impl Embedder {
    pub fn initialize(model_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let env = ort::init()
            .with_name("EmbedSearch")
            .build()?;
        let session = ort::session::Session::builder(&env)?.commit_from_file(model_path)?;
    
        Ok(Embedder {
            session
        })
    }
}
