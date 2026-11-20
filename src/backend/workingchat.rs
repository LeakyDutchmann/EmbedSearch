use grammers_session::types::PeerRef;
use std::collections::HashMap;
use crate::backend::embedding::IndexedImage;
use crate::backend::types::{ChatId, MessageId};
use crate::backend::workingchat::ChatContents::Images;

#[derive(Debug, PartialEq, Eq)]
pub struct Chat {
    pub chat_id: ChatId,
    pub name: String,
    pub peer_ref: PeerRef,
    pub topics: Option<Vec<Topic>>,
}


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TopicId {
    pub id: i64
}

#[derive(Debug, PartialEq, Eq)]
pub struct Topic {
    pub name: String,
    pub id: TopicId,
}

impl Topic {
    pub fn from_key_val(k: &TopicId, val: &WorkingTopic) -> Self {
        Topic {
            id: k.clone(),
            name: val.name.clone(),
        }
    }
}


pub struct WorkingTopic {
    pub name: String,
    pub indexed_images: HashMap<MessageId, IndexedImage>,
}

impl WorkingTopic {
    fn from_topic(t: &Topic) -> Self {
        WorkingTopic {
            name:  t.name.clone(), 
            indexed_images: HashMap::new()
        }
    }
}


pub enum ChatContents {
    Images(HashMap<MessageId, IndexedImage>),
    Topics(HashMap<TopicId, WorkingTopic>),
}


pub struct WorkingChat {
    pub name: String,
    pub peer_ref: PeerRef,
    pub contents: ChatContents,
}

impl WorkingChat {
    pub fn from_chat(chat: Chat) -> Self {
        let contents = if let Some(topics) = chat.topics {
            let mut tps = HashMap::new();
            for t in topics {
                let id = t.id.clone();
                tps.insert(id, WorkingTopic::from_topic(&t));
            }
            ChatContents::Topics(tps)
        } else {
            ChatContents::Images(HashMap::new())
        };
        
        WorkingChat {
            name: chat.name,
            peer_ref: chat.peer_ref,
            contents,
        }
    }
}


