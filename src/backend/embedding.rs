use grammers_client::{Client, media::Media};
use grammers_session::Session;
use grammers_session::storages::SqliteSession;
use grammers_mtsender::SenderPool;
use image::{DynamicImage, GenericImageView};
use ndarray::{Array4, Array2};
use ort::environment::Environment;
use ort::value::{Tensor, Value};
use image::ImageReader;

use crate::backend::workingchat::ChatId;


pub struct IndexedImage {
    pub msg_id: i32,
    pub chat_id: ChatId,
    pub embedding: Box<[f32]>,
}

pub fn cosine_similarity(emb_1: &[f32], emb_2: &[f32]) -> f32 {
    let dot: f32 = emb_1.iter()
        .zip(emb_2)
        .map(|(x, y)| x * y)
        .sum();

    let norm_1: f32 = emb_1.iter()
        .map(|x| x * x)
        .sum::<f32>()
        .sqrt();

    let norm_2: f32 = emb_2.iter()
        .map(|y| y * y)
        .sum::<f32>()
        .sqrt();

    dot / (norm_1 * norm_2)
}

pub fn preprocess_img(img: DynamicImage) -> Array4<f32>{

    let (w, h) = img.dimensions();
    println!("dimensions: width: {}, height: {}", w, h);

    let scale_factor = 224.0 / w.min(h) as f32;

    let new_w = (w as f32 * scale_factor).round() as u32;
    let new_h = (h as f32 * scale_factor).round() as u32;

    println!("resizing to: width {} height {}", new_w, new_h);
    
    let img = img.resize_exact(
        new_w,
        new_h,
        image::imageops::FilterType::Lanczos3
    );
    let crop_x = (img.width() - 224) / 2;
    let crop_y = (img.height() - 224) /2;

    let img = img.crop_imm(crop_x, crop_y, 224, 224);
    let rgb = img.to_rgb8();


    let mut tensor = Array4::zeros((1, 3, 224, 224));

    for y in 0..224 {
        for x in 0..224 {
            let pixel = rgb.get_pixel(x, y);
            let r = pixel[0] as f32 /255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            
            tensor[[0, 0, y as usize, x as usize]] = (r - 0.48145466) / 0.26862954;

            tensor[[0, 1, y as usize, x as usize]] = (g - 0.4578275) / 0.26130258;

            tensor[[0, 2, y as usize, x as usize]] = (b - 0.40821073) / 0.27577711;

        }
    }
    tensor 
}