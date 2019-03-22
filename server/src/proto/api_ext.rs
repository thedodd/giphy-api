use crate::proto::api::{
    response_frame::Response,
    ResponseFrame, LoginResponse,
};

impl ResponseFrame {
    /// Create a new login response.
    pub fn login(id: String, res: LoginResponse) -> Self {
        let response = Some(Response::Login(res));
        ResponseFrame{id, response}
    }
}
