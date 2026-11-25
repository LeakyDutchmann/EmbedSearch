use crate::backend::types::{ChatId, TopicId};
use crate::backend::protocol::ChatInfo;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UiChat {
    pub id: ChatId,
    pub name: String,
    pub topics: Option<Vec<UiTopic>>,
}

impl UiChat {
    pub fn from_info(info: ChatInfo) -> Self {
        let topics = {
            if let Some(t) = info.topics {
                let mut t_vec = Vec::new();
                for topic in t {
                    t_vec.push(UiTopic {
                        id: topic.id,
                        name: topic.name
                    });
                }
                Some(t_vec)
            } else {
                None
            }
        };
        
        UiChat {
            id: info.chat_id,
            name: info.name,
            topics,
        }
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UiTopic {
    pub name: String,
    pub id: TopicId
}


pub enum UiRow<'a> {
    Chat(&'a UiChat),
    Row(&'a UiChat, &'a Vec<UiTopic>)
}