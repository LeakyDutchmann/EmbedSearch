pub mod state;
pub mod view;

use crate::backend::protocol::{Pending, InitializeResponse, BackEnd};
use super::task::drive;

enum AuthState {
    Checking(Option<Pending<bool>>),
    EnterPhone,
    RequestingCode(Option<Pending<()>>),
    EnterCode,
    SigningIn(Option<Pending<()>>),
    Authorized(Option<Pending<InitializeResponse>>),
    Error(String),
}


pub struct AuthFlow {
    state: AuthState,
    phone: Option<String>,
    code: Option<String>,
    input: String,
}

impl AuthFlow {
    pub fn init() -> Self {
        AuthFlow {
            state: AuthState::Checking(None),
            phone: None,
            code: None,
            input: String::new(),
        }
    }
}