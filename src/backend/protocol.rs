use std::any;

use anyhow::anyhow;
use tokio::task::JoinHandle;
use std::collections::{HashMap, HashSet};

use backend::search::SearchEngine;
use crate::backend;
use backend::types::{ChatId, TopicId};
use backend::workingchat::{Chat, Topic};
use tokio::sync::oneshot;


pub struct ChatInfo {
    pub chat_id: ChatId,
    pub name: String,
    pub topics: Option<Vec<Topic>>
}


pub struct InitializeResponse {
    pub indexed_chats: Vec<ChatInfo>,
    pub available_chats: Vec<ChatInfo>,
}


pub struct BackEnd {
    sender: tokio::sync::mpsc::Sender<Request>,
}


pub enum Request {
    ReqLoginCode{
        phone: String,
        feedback: oneshot::Sender::<anyhow::Result<()>>,
    },
    SignIn{
        code: String,
        feedback: oneshot::Sender::<anyhow::Result<()>>,
    },
    IsAuthorized {
        feedback: oneshot::Sender<anyhow::Result<bool>>,
    },
    InitializeAppData {
        feedback: oneshot::Sender<anyhow::Result<InitializeResponse>>
    },
    RefreshAppData {
        feedback: oneshot::Sender<anyhow::Result<InitializeResponse>>
    },
    GetChats,
    SetWorkingChats{chats: HashMap<ChatId, Option<HashSet<TopicId>>>, feedback: oneshot::Sender<anyhow::Result<()>>},
    LoadIndexed,
    IndexChats{feedback: oneshot::Sender<anyhow::Result<()>>},
    SearchImage,
    GetImages,
    
}


pub struct Pending<T> {
    receiver: oneshot::Receiver<anyhow::Result<T>>,
}

impl<T> Pending<T> {
    pub fn try_take(&mut self) -> Option<anyhow::Result<T>> {
        if let Ok(val) = self.receiver.try_recv() {
            match val {
                Ok(v) => return Some(Ok(v)),
                Err(e) => {
                    println!("Backend operation failed: {e}");
                    return Some(Err(anyhow!("Backend operation failed: {e}")));
                }
            }
        }
        None
    }
}


impl BackEnd {
    pub fn initialize() -> Self {
        
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Request>(4096);

        let task = tokio::spawn(async move {
            let mut engine = SearchEngine::initialize().await.unwrap();
            
            while let Some(req) = rx.recv().await {
                match req {
                    Request::ReqLoginCode { phone, feedback} => {
                        let res = engine.request_login_code(phone).await;
                        
                        println!("request_login_code result: {:?}", res);
                        
                        feedback.send(res);
                        
                    },
                    Request::SignIn { code , feedback} => {
                        let res = engine.sign_in(&code).await;
                        feedback.send(res);
                    },
                    Request::IsAuthorized{feedback} => {
                        let res = engine.is_authorized().await;
                        feedback.send(res);
                    },
                    Request::RefreshAppData { feedback } => {
                        let result: anyhow::Result<InitializeResponse> = async {
                            let w_chats = engine.get_working_chats();
                            let mut chat_info = Vec::new();
                            for chat in w_chats {
                                chat_info.push(ChatInfo {
                                    chat_id: chat.chat_id,
                                    name: chat.name,
                                    topics: chat.topics
                                })
                            }

                            let all_chats = engine.get_available_chats().await?;
                            let mut a_chats = Vec::new();
                            for chat in all_chats {
                                a_chats.push(ChatInfo {
                                    chat_id: chat.chat_id,
                                    name: chat.name,
                                    topics: chat.topics
                                })
                            }
                            Ok(InitializeResponse {
                                indexed_chats: chat_info,
                                available_chats: a_chats,
                            })
                        }.await;
                        feedback.send(result);
                        
                    },
                    Request::SetWorkingChats { chats, feedback } => {
                        let result: anyhow::Result<()> = async {
                            let available = engine.get_available_chats().await?;
                            let mut selected: Vec<Chat> = available.into_iter()
                                .filter(|c| chats.contains_key(&c.chat_id))
                                .collect();
                            for c in selected.iter_mut() {
                                if let Some(Some(s_topics)) = chats.get(&c.chat_id) {
                                    if let Some(a_topics) = &mut c.topics {
                                        a_topics.retain(|t| s_topics.contains(&t.id));
                                    }
                                }
                            }
                            engine.set_working_chats(selected);
                            Ok(())
                        }.await;
                        feedback.send(result);
                    }, 
                    Request::IndexChats { feedback } => {
                        let result = engine.index_chats().await;
                        feedback.send(result);
                    },
                    Request::InitializeAppData { feedback } => {
                        let result: anyhow::Result<InitializeResponse> = async {
                            engine.restore_environment().await?;
                            
                            engine.load_indexed().await?;

                            let w_chats = engine.get_working_chats();
                            let mut chat_info = Vec::new();
                            for chat in w_chats {
                                chat_info.push(ChatInfo {
                                    chat_id: chat.chat_id,
                                    name: chat.name,
                                    topics: chat.topics,
                                })
                            }

                            let all_chats = engine.get_available_chats().await?;
                            let mut a_chats = Vec::new();
                            for chat in all_chats {
                                a_chats.push(ChatInfo {
                                    chat_id: chat.chat_id,
                                    name: chat.name,
                                    topics: chat.topics,
                                })
                            }
                            Ok(InitializeResponse {
                                indexed_chats: chat_info,
                                available_chats: a_chats,
                            })
                        }.await;
                        feedback.send(result);
                    }
                    _ => {
                        
                    }
                }
            }
        });

        BackEnd {sender: tx}
    }
    pub fn is_authorized(&mut self) -> anyhow::Result<Pending<bool>>{
        let (rp_tx, rx) = tokio::sync::oneshot::channel();

        self.sender
            .try_send(Request::IsAuthorized { feedback: rp_tx })?;
        
        Ok(Pending { receiver: rx })
    }
    pub fn request_login_code(&self, phone: String) -> anyhow::Result<Pending<()>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::ReqLoginCode { phone, feedback: rp_tx})?;
        Ok(Pending{receiver: rx})
    }
    pub fn sign_in(&self, code: String) -> anyhow::Result<Pending<()>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::SignIn { code, feedback: rp_tx })?;
        Ok(Pending{receiver: rx})
    }
    pub fn get_app_data(&self) -> anyhow::Result<Pending<InitializeResponse>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::RefreshAppData { feedback: rp_tx })?;
        Ok(Pending{receiver: rx})
    }
    pub fn restore_environment(&self) -> anyhow::Result<Pending<InitializeResponse>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::InitializeAppData { feedback: rp_tx })?;
        Ok(Pending{receiver: rx})
    }
    pub fn set_working_chats(&self, chats: HashMap<ChatId, Option<HashSet<TopicId>>>) -> anyhow::Result<Pending<()>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::SetWorkingChats {feedback: rp_tx, chats: chats})?;
        Ok(Pending { receiver: rx })
    }
    pub fn index_chats(&self) -> anyhow::Result<Pending<()>> {
        let (rp_tx, rx) = oneshot::channel();
        self.sender.try_send(Request::IndexChats{feedback: rp_tx})?;
        Ok(Pending {receiver: rx })
    }
 }