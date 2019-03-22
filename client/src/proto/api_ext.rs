use uuid::Uuid;

use crate::proto::api::{
    request_frame::Request,
    LoginRequest, RequestFrame,
};

impl RequestFrame {
    /// Create a new login request.
    pub fn login(req: LoginRequest) -> Self {
        let id = Uuid::new_v4().to_simple().to_string();
        let request = Some(Request::Login(req));
        RequestFrame{id, request}
    }
}
