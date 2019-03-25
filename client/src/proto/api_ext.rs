use uuid::Uuid;

use crate::proto::api::{
    request_frame::Request,
    LoginRequest, RegisterRequest, RequestFrame, SearchGiphyRequest,
};

impl RequestFrame {
    /// Create a new register request.
    pub fn register(email: String, password: String) -> Self {
        let id = Uuid::new_v4().to_simple().to_string();
        let request = Some(Request::Register(RegisterRequest{email, password}));
        RequestFrame{id, request}
    }

    /// Create a new login request.
    pub fn login(email: String, password: String) -> Self {
        let id = Uuid::new_v4().to_simple().to_string();
        let request = Some(Request::Login(LoginRequest{email, password}));
        RequestFrame{id, request}
    }

    /// Create a new request for searching the Giphy API.
    pub fn search_giphy(query: String, jwt: String) -> Self {
        let id = Uuid::new_v4().to_simple().to_string();
        let request = Some(Request::SearchGiphy(SearchGiphyRequest{jwt, query}));
        RequestFrame{id, request}
    }
}
