
use anyhow::anyhow;
use crate::backend::{self, workingchat::{ChatContents, Topic}};
use grammers_client::{Client, media::Media};
use grammers_session::storages::SqliteSession;
use grammers_mtsender::SenderPool;
use image::DynamicImage;
use std::path::Path;
use std::sync::Arc;
use grammers_client::client::LoginToken;
use grammers_client::peer::Peer;
use tl::types::Channel;
use tl::enums::Chat as TlChat;
use grammers_session::types::PeerRef;
use backend::workingchat::{Chat, WorkingChat};
use std::collections::HashMap;
use backend::types::{ChatId, PendingImage, MessageId, TopicId};



use grammers_tl_types as tl;
use tl::enums::messages::Messages;


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
    pub async fn get_available_chats(&self) -> anyhow::Result<Vec<Chat>> {
        let mut chats: Vec<Chat> = Vec::new();

        let mut dialogs = self.client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            
            let peer = dialog.peer();
            let peer_ref = match peer.to_ref().await {
                Ok(Some(p_r)) => p_r,
                _ => {return Err(anyhow!("Failed to get peer reference"))}
            };

            let topics = {
                match peer {
                    Peer::Group(g) => {
                        if g.is_megagroup() {
                            match &g.raw {
                                TlChat::Channel(ch) => {  
                                    if ch.forum {
                                        println!("Got channel-megagroup with topics: {}", g.title().unwrap());
                                        let tps = self.get_topics(&peer_ref).await?;
                                        Some(tps)
                                    } else {
                                        None
                                    }
                                },
                                _ => {
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            };
            
            chats.push(Chat {
                chat_id: ChatId::from_peer_ref(peer_ref.clone())?,
                peer_ref,
                name: peer.name().unwrap_or("Unknown").to_string(),
                topics,
            });
            
        }
        Ok(chats)
    }
    pub async fn get_unindexed_images(&mut self, chat: &WorkingChat) -> anyhow::Result<Vec<PendingImage>> {
        match &chat.contents {
            ChatContents::Images(indexed_images) => {
                let mut messages = self.client.iter_messages(chat.peer_ref);
                           
                let mut images = Vec::new();
                let mut bytes = Vec::new();
                let mut x = 0;
                while let Some(msg) = messages.next().await? {
                    let Some(media) = msg.media() else {
                        continue 
                    };
        
                    let msg_id = MessageId{id: msg.id()};
                    
                    if indexed_images.contains_key(&msg_id) {
                        continue;
                    }
                    
                    match media {
                        Media::Photo(photo) => {
                            bytes.clear();
                            
                            let mut download = self.client.iter_download(&photo);
                
                            while let Some(chunk) = download.next().await? {
                                bytes.extend(chunk);
                            }
                            x += 1;
                            println!("Loaded {} images", x);
                            let img = image::load_from_memory(&bytes)?;
                            images.push(PendingImage {
                                img,
                                msg_id,
                                topic_id: None,
                            });             
                        },
                        _ => continue
                    }
                }
                println!("Got {} images from chat: {}", images.len(), chat.name);
                Ok(images)
            }, 
            ChatContents::Topics(topics) => {
                let mut images = Vec::new();
                let mut bytes = Vec::new();
                let mut x = 0;

                for (topic_id, topic) in topics {
                    println!("Lading from topic: {}", topic.name);
                    let peer = chat.peer_ref;
                    let mut ids = vec![];
                    let mut offset_id = 0;

                    let mut x = 0;
                    
                    loop {
                        let page = self.fetch_page_mtproto(offset_id, &peer, &topic_id).await?;
                        for msg in page.iter() {
                            ids.push(msg.id);
                        }
                        let Some(new_offset) = page.iter().map(|m| m.id).min() else {
                            break;
                        };
                        offset_id = new_offset;
                    }
                    
                    let messages = self.client.get_messages_by_id(peer, &ids).await?;
                    for opt in messages {
                        if let Some(msg) = opt {
                            let Some(media) = msg.media() else {
                                continue 
                            };
                
                            let msg_id = MessageId{id: msg.id()};
                            
                            if topic.indexed_images.contains_key(&msg_id) {
                                continue;
                            }
                            
                            match media {
                                Media::Photo(photo) => {
                                    bytes.clear();
                                    
                                    let mut download = self.client.iter_download(&photo);
                        
                                    while let Some(chunk) = download.next().await? {
                                        bytes.extend(chunk);
                                    }
                                    x += 1;
                                    println!("Loaded {} images", x);
                                    let img = image::load_from_memory(&bytes)?;
                                    images.push(PendingImage {
                                        img,
                                        msg_id,
                                        topic_id: Some(topic_id.clone())
                                    });         
                                    x += 1;
                                },
                                _ => continue
                            }
                        }
                    }
                    println!("Got {} images from topic {}", images.len(), chat.name.clone() + "/" + &topic.name);
                }
                println!("Got {} images from topics in {}", images.len(), chat.name);
                Ok(images)
            }
        } 
    }
    pub async fn fetch_page_mtproto(&self, offset_id: i32, peer: &PeerRef, topic_id: &TopicId) -> anyhow::Result<Vec<tl::types::Message>> {
        let messages = self.client.invoke(&tl::functions::messages::Search{
            q: String::new(),
            peer: peer.into(),
            filter: tl::enums::MessagesFilter::InputMessagesFilterPhotos,
            top_msg_id: Some(topic_id.id as i32),
            offset_id,
            add_offset: 0,
            limit: 100,
            max_id: 0,
            min_id: 0,
            hash: 0,
            from_id: None,
            saved_peer_id: None,
            saved_reaction: None,
            max_date: 0,
            min_date: 0,
        }).await?;
        
        let mut msg_vec = vec![];
        match messages {
            Messages::Messages(m) => {
                println!("Got Messages::Messages");
                for m in m.messages {
                    match m {
                        tl::enums::Message::Message(msg) => {
                            msg_vec.push(msg)
                        }
                        _ => {
                            continue;
                        }
                    }
                }
            }, 
            Messages::ChannelMessages(m) => {
                println!("Got Messages::ChannelMessages");
                for m in m.messages {
                    match m {
                        tl::enums::Message::Message(msg) => {
                            msg_vec.push(msg);
                        },
                        _ => {
                            continue
                        }
                    }
                }
            }
            Messages::NotModified(m) => {
                println!("Got {} Messages::NotModified", m.count);
            },
            Messages::Slice(m) => {
                println!("Got slice with {} messages", m.count);
                for m in m.messages {
                    match m {
                        tl::enums::Message::Message(msg) => {
                            msg_vec.push(msg);
                        },
                        _ => {
                            continue
                        }
                    }
                }
            }
        }
        Ok(msg_vec)
    }
    pub async fn download_images(&self, peer_ref: PeerRef, msg_ids: &[MessageId] ) -> anyhow::Result<HashMap<MessageId, DynamicImage>> {
        let message_ids: Vec<i32> = msg_ids.iter().map(|s| s.id).collect();
        let messages = self.client.get_messages_by_id(peer_ref, &message_ids).await?;

        let messages: Vec<grammers_client::message::Message> = messages.into_iter().filter(|m| m.is_some()).map(|m| m.unwrap()).collect();
        let count = message_ids.len() - messages.len();
        println!("Lost {} messages", count);

        let mut images: HashMap<MessageId, DynamicImage> = HashMap::new();
        
        let mut bytes: Vec<u8> = Vec::new();
        for msg in messages {
            let Some(media) = msg.media() else {
                continue;
            };

            let id = msg.id();
            
            bytes.clear();
            match media {
                Media::Photo(photo) => {
                    let mut download = self.client.iter_download(&photo);

                    while let Some(chunk) = download.next().await? {
                        bytes.extend(chunk);
                    }
                    
                    let Ok(img) = image::load_from_memory(&bytes) else {
                        continue;
                    };
                    
                    images.insert(MessageId { id }, img);
                },
                _ => {
                    continue;
                }
            }
        }
        Ok(images)
    }
    pub async fn get_topics(&self, peer_ref: &PeerRef) -> anyhow::Result<Vec<Topic>> {
        let peer: tl::enums::InputPeer = peer_ref.into();

        let topics = self.client.invoke(
            &tl::functions::messages::GetForumTopics {
                peer: peer,
                offset_date: 0,
                offset_id: 0,
                offset_topic: 0,
                limit: 100,
                q: None,
            }
        ).await?;

        match topics {
            tl::enums::messages::ForumTopics::Topics(t) => {
                let mut f_topics = Vec::new();
                for topic in t.topics {
                    match topic {
                        tl::enums::ForumTopic::Topic(top) => {
                            let t = Topic {
                                name: top.title,
                                id: TopicId{ id: top.id as i64}
                            };
                            f_topics.push(t);
                        },
                        _ => {
                            continue;
                        }
                    }
                }
                return Ok(f_topics);
            }
        }
    }
}

