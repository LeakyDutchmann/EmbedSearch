use super::{AuthFlow, AuthState, BackEnd, InitializeResponse, drive};

impl AuthFlow {
    fn step(&mut self, backend: &mut BackEnd) -> anyhow::Result<Option<InitializeResponse>> {
        match &mut self.state {
            AuthState::Checking(p) => { 
                if let Some(authorised) = drive(p, || backend.is_authorized()) {
                    if authorised? {
                        self.state = AuthState::Authorized(None);
                    } else {
                        self.state = AuthState::EnterPhone;
                    }
                }
                return Ok(None);
            }
            AuthState::EnterPhone => { 
                return Ok(None);
            }
            AuthState::RequestingCode(p) => { 
                let phone = self.phone.as_ref().unwrap().clone();
                if let Some(res) = drive(p, || backend.request_login_code(phone)) {
                    res?;
                    self.state = AuthState::EnterCode;
                }
                return Ok(None);
            }
            AuthState::EnterCode => { 
                return Ok(None);
            }
            AuthState::SigningIn(p) => { 
                let code = self.code.as_ref().unwrap().clone();
                if let Some(res) = drive(p, ||backend.sign_in(code)) {
                    res?;
                    self.state = AuthState::Authorized(None)
                }
                return Ok(None);
            }
            AuthState::Authorized(p_data) => { 
                if let Some(init_r) = drive(p_data, || backend.restore_environment()) {
                    return Ok(Some(init_r?))
                }
                return Ok(None);
            }
            AuthState::Error(_) => {
                return Ok(None);
            }
        }
    }
    pub fn update(&mut self, backend: &mut BackEnd) -> Option<InitializeResponse> {
        match self.step(backend) {
            Ok(opt) => opt,
            Err(e) => {
                self.state = AuthState::Error(e.to_string());
                None
            }
        }
    }
}