use crate::backend::{
    self, database::Database, embedding::{DbIndexedImage, Embedder, IndexedImage}, telegram::TelegramClient, types::{ChatId, MessageId, TopicId}, workingchat::{Chat, ChatContents, Topic, WorkingChat, WorkingTopic}
};

use anyhow::anyhow;
use grammers_client::client::LoginToken;
use image::DynamicImage;
use std::collections::HashMap;

pub struct SearchEngine {
    embedder: Embedder,
    telegram_client: TelegramClient,
    database: Database,
    working_chats: HashMap<ChatId, WorkingChat>,
    login_token: Option<LoginToken>,
}

pub struct SearchResult {
    chat_id: ChatId,
    msg_id: MessageId,
    similarity: f32,
}

impl SearchEngine {
    pub async fn initialize() -> anyhow::Result<Self>{
        dotenvy::dotenv().ok(); 
        let api_id: i32 = std::env::var("TELEGRAM_API_ID")?.parse()?;
        
        
        let embedder = backend::embedding::Embedder::initialize("models/model.onnx")?;
        let telegram_client = TelegramClient::intialize("session/session.db", api_id).await?;
        let database = Database::initialize("data/database.db").await?;

        Ok(SearchEngine {
            embedder,
            telegram_client,
            working_chats: HashMap::new(),
            database,
            login_token: None,
        })
    }
    pub async fn restore_environment(&mut self) -> anyhow::Result<()>{
        let chat_ids = self.database.get_indexed_chat_ids().await?;

        let chats = self.telegram_client.get_available_chats().await?;

        let filtered_chats: Vec<Chat> = chats.into_iter().filter(|c| chat_ids.contains(&c.chat_id)).collect();
        for chat in filtered_chats.into_iter() {
            let id = chat.chat_id.clone();
            let w_chat = WorkingChat::from_chat(chat);
            self.working_chats.insert(id, w_chat);
        }
        
        Ok(())
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
    pub fn get_working_chats(&self) -> Vec<Chat> {
        let mut chats = Vec::new();
        
        for (id, chat) in self.working_chats.iter() {
            let topics = match &chat.contents {
                ChatContents::Topics(t) => {
                    Some(t.iter().map(|(k, v)| Topic::from_key_val(&k, &v)).collect::<Vec<Topic>>())
                },
                _ => {
                    None
                }
            };
            let chat = Chat {
                name: chat.name.clone(),
                chat_id: id.clone(),
                peer_ref: chat.peer_ref,
                topics,
            };
            chats.push(chat);
        }
        chats
    }
    pub fn set_working_chats(&mut self, mut chats: Vec<Chat>) {
        while let Some(chat) = chats.pop() {
            println!("Setting {:?} as wroking chat", chat);
            if let Some(w_chat) = self.working_chats.get_mut(&chat.chat_id) {
                match &mut w_chat.contents {
                    ChatContents::Images(_) => {
                        continue;
                    },
                    ChatContents::Topics(w_topics) => {
                        if let Some(topics) = chat.topics {
                            for topic in topics {
                                if !w_topics.contains_key(&topic.id) {
                                    let id = topic.id.clone();
                                    w_topics.insert(id, WorkingTopic::from_topic(&topic));
                                }
                            }
                        }
                    }
                }
            } else {
                self.working_chats.insert(chat.chat_id.clone(), WorkingChat::from_chat(chat));
            }
        }
    }
    pub async fn load_indexed(&mut self) -> anyhow::Result<()> {
        let indexed = self.database.load_indexed().await?;

        for img in indexed {
            if let Some(chat) = self.working_chats.get_mut(&img.chat_id) {
                match &mut chat.contents {
                    ChatContents::Images(map) => {
                        map.insert(img.msg_id, IndexedImage {embedding: img.embedding});
                    },
                    ChatContents::Topics(topics) => {
                        let Some(t_id) = img.topic_id else {
                            continue;
                        };
                        if let Some(topic) = topics.get_mut(&t_id) {
                            topic.indexed_images.insert(img.msg_id, IndexedImage {embedding: img.embedding});
                        }
                    }
                }
                
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
                    topic_id: raw.topic_id
                });
            }

            self.database.save_indexed(&indexed).await?;

            for img in indexed {
                match &mut chat.contents {
                    ChatContents::Images(indexed_images) => {
                        indexed_images.insert(img.msg_id.clone(), IndexedImage { embedding: img.embedding });
                    }, 
                    ChatContents::Topics(topics) => {
                        let Some(t_id) = img.topic_id else {
                            continue
                        };
                        if let Some(topic) = topics.get_mut(&t_id) {
                            topic.indexed_images.insert(img.msg_id.clone(), IndexedImage { embedding: img.embedding });
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn search_image(&mut self, img: DynamicImage, chat_id: ChatId, topic_id: Option<TopicId>) -> anyhow::Result<Vec<SearchResult>> {
        let embeddding = self.embedder.create_embedding(img)?;

        let Some(working_chat) = self.working_chats.get(&chat_id) else {
            return Err(anyhow!("Failed to search image: CHAT NOT FOUND"))
        };
        
        let mut result = Vec::new();

        match &working_chat.contents {
            ChatContents::Images(indexed_images) => {
                for (msg_id, indexed) in indexed_images {
                    let similarity = cosine_similarity(&embeddding, &indexed.embedding);
                    result.push(SearchResult {
                        chat_id: chat_id.clone(),
                        msg_id: msg_id.clone(),
                        similarity 
                    });
                }
        
                result.sort_by(|a, b| {
                    b.similarity
                        .partial_cmp(&a.similarity)
                        .unwrap()
                });
        
                result.retain(|a| a.similarity > 0.5);
        
                
            },
            ChatContents::Topics(topics) => {
                let Some(t_id) = topic_id else {
                    return Err(anyhow::anyhow!("Forum topic not found"));
                };
                if let Some(topic) = topics.get(&t_id) {
                    for (msg_id, indexed) in &topic.indexed_images {
                        let similarity = cosine_similarity(&embeddding, &indexed.embedding);
                        result.push(SearchResult {
                            chat_id: chat_id.clone(),
                            msg_id: msg_id.clone(),
                            similarity 
                        });
                    }
            
                    result.sort_by(|a, b| {
                        b.similarity
                            .partial_cmp(&a.similarity)
                            .unwrap()
                    });
            
                    result.retain(|a| a.similarity > 0.5);
                }
            }
        }
        Ok(result)
    }
    pub async fn get_images(&self, chat_id: ChatId, msg_ids: &[MessageId]) -> anyhow::Result<HashMap<MessageId, DynamicImage>> {
        let Some(working_chat) = self.working_chats.get(&chat_id) else { 
            return Err(anyhow!("Failed to upload image. CHAT NOT FOUND"))
        };
        
        let images = self.telegram_client.download_images(working_chat.peer_ref.clone(), msg_ids).await?;
        Ok(images)
    }
}

fn cosine_similarity(emb_1: &[f32], emb_2: &[f32]) -> f32 {
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