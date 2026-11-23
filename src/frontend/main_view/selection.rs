use std::collections::{HashMap, HashSet};
use super::{ChatId, TopicId};
use super::types::UiTopic;

pub struct Selection {
    selected_chats: HashSet<ChatId>,
    selected_topics: HashMap<ChatId, HashSet<UiTopic>>,
}

impl Selection {
    pub fn is_chat_selected(&self, id: &ChatId) -> bool {
        if let Some(_) = &self.selected_chats.get(id) {
            true
        } else {
            false
        }
    }
    pub fn is_topic_selected(&self, chat_id: &ChatId, topic_id: &TopicId) -> bool {
        if let Some(topics) = self.selected_topics.get(chat_id) {
            for t in topics {
                if &t.id == topic_id {
                    return true;
                }
            }
        } 
        return false;
    }
    pub fn get_selected_chats(&self) -> HashMap<ChatId, Option<HashSet<TopicId>>> {
        let mut chats = HashMap::new();
        for chat in self.selected_chats.iter() {
            if let Some(topics) = self.selected_topics.get(&chat) {
                let topics: HashSet<TopicId> = topics.iter().map(|v| v.id.clone()).collect();
                chats.insert(chat.clone(), Some(topics.clone()));
            } else {
                chats.insert(chat.clone(), None);
            }
        }
        println!("Sending selected chats: {:?}", chats);
        chats
    }
    pub fn clear(&mut self) {
        self.selected_chats.clear();
        self.selected_topics.clear();
    }
}