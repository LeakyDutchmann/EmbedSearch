use std::any;

use anyhow::anyhow;
use tokio::task::JoinHandle;

use crate::backend::search::SearchEngine;
use tokio::sync::oneshot;


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
    GetChats,
    SetChats,
    LoadIndexed,
    IndexChats,
    SearchImage,
    GetImages,
    
}


pub struct Pending<T> {
    receiver: oneshot::Receiver<anyhow::Result<T>>,
}

impl<T> Pending<T> {
    pub fn try_take(&mut self) -> Option<T> {
        if let Ok(val) = self.receiver.try_recv() {
            match val {
                Ok(v) => return Some(v),
                Err(e) => {
                    println!("Backend operation failed: {e}");
                    return None;
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
}