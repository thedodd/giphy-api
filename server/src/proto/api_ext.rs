use std::collections::HashMap;

use crate::proto::api::{self, response_frame::Response};

impl api::Error {
    const ISE: &'static str = "Internal server error.";

    /// Create a new instance.
    pub fn new(description: &str, status: u32, code: Option<&str>, meta: Option<HashMap<String, String>>) -> Self {
        let description = description.to_string();
        let code = code.map(|s| s.to_string()).unwrap_or_default();
        let meta = meta.unwrap_or_default();
        Self{description, status, code, meta}
    }

    /// Create a new instance representing an internal server error.
    pub fn new_ise(code: Option<&str>, meta: Option<HashMap<String, String>>) -> Self {
        Self::new(Self::ISE, 500, code, meta)
    }
}

impl api::ResponseFrame {
    /// Create a new error response.
    pub fn error(id: Option<&str>, error: api::Error) -> Self {
        let id = id.map(|s| s.to_string()).unwrap_or_default();
        Self{id, error: Some(error), response: None}
    }

    /// Create a new register response.
    pub fn register(id: String, user: String, email: String, jwt: String) -> Self {
        let response = Some(Response::Register(api::RegisterResponse{id: user, email, jwt}));
        Self{id, error: None, response}
    }

    /// Create a new login response.
    pub fn login(id: String, user: String, email: String, jwt: String) -> Self {
        let response = Some(Response::Login(api::LoginResponse{id: user, email, jwt}));
        Self{id, error: None, response}
    }
}
