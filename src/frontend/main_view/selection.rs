use std::collections::{HashMap, HashSet};
use super::{ChatId, TopicId};
use super::types::UiTopic;

pub struct Selection {
    pub selected_chats: HashSet<ChatId>,
    pub selected_topics: HashMap<ChatId, HashSet<UiTopic>>,
}

impl Selection {
    pub fn init() -> Self {
        Selection { 
            selected_chats: HashSet::new(),
            selected_topics: HashMap::new()
        }
    }
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
    pub fn remove_chat(&mut self, id: &ChatId) {
        self.selected_chats.remove(id);
    }
    pub fn remove_topic(&mut self, chat_id: &ChatId, tp: &UiTopic) {
        if let Some(topics) = self.selected_topics.get_mut(chat_id) {
            topics.remove(tp);
            if topics.is_empty() {
                self.selected_topics.remove(chat_id);
                self.selected_chats.remove(chat_id);
            }
        }
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
    pub fn select_chat(&mut self, id: &ChatId) {
        self.selected_chats.insert(id.clone());
    }
    pub fn toggle_topic(&mut self, ch_id: &ChatId, topic: &UiTopic) {
        let set = self.selected_topics.entry(ch_id.clone()).or_default();

        if !set.remove(topic) {
            set.insert(topic.clone());
        }

        if set.is_empty() {
            self.selected_topics.remove(ch_id);
            self.selected_chats.remove(ch_id);
        } else {
            self.selected_chats.insert(ch_id.clone());
        }
    }
}