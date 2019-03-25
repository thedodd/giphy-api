use std::collections::HashMap;

use crate::proto::api::{self, response_frame::Response};


/// An error variant which represents a high-level error or a domain specific error.
///
/// If an ISE takes place, authN failure or the like, use `ErrSystem(_)`. If the error is domain
/// specific, use `ErrDomain(_)`.
pub enum Error {
    ErrSystem(api::ErrorResponse),
    ErrDomain(api::ErrorDomain),
}

impl api::ErrorDomain {
    /// Create a new domain error instance.
    pub fn new(description: &str, fields: Option<HashMap<String, String>>) -> Self {
        let description = description.to_string();
        let fields = fields.unwrap_or_default();
        Self{description, fields}
    }
}

impl api::ErrorResponse {
    const ISE: &'static str = "Internal server error.";
    const AUTHN: &'static str = "Authentication failed. Invalid credentials provided.";
    const INVALID: &'static str = "Invalid or malformed frame received.";

    /// Create a new instance.
    fn new(description: &str, etype: api::ErrorResponseType) -> Self {
        let description = description.to_string();
        Self{description, etype: etype.into()}
    }

    /// Create a new instance representing an internal server error.
    pub fn new_ise() -> Self {
        Self::new(Self::ISE, api::ErrorResponseType::EtypeIse)
    }

    /// Create a new instance authentication failure instance.
    pub fn new_authn() -> Self {
        Self::new(Self::AUTHN, api::ErrorResponseType::EtypeAuthn)
    }

    /// Create a new instance representing request validity failure.
    pub fn new_invalid() -> Self {
        Self::new(Self::INVALID, api::ErrorResponseType::EtypeInvalid)
    }
}

impl api::ResponseFrame {
    /// Create a new error response.
    pub fn error(id: Option<&str>, error: api::ErrorResponse) -> Self {
        let id = id.map(|s| s.to_string()).unwrap_or_default();
        Self{id, response: Some(Response::Error(error))}
    }

    /// Create a new register response.
    pub fn register(id: String, user: String, email: String, jwt: String) -> Self {
        let response = Some(Response::Register(api::RegisterResponse{error: None, id: user, email, jwt}));
        Self{id, response}
    }

    /// Create a new register response error.
    pub fn register_err(id: String, error: api::ErrorDomain) -> Self {
        let mut inner = api::RegisterResponse::default();
        inner.error = Some(error);
        let response = Some(Response::Register(inner));
        Self{id, response}
    }

    /// Create a new login response.
    pub fn login(id: String, user: String, email: String, jwt: String) -> Self {
        let response = Some(Response::Login(api::LoginResponse{error: None, id: user, email, jwt}));
        Self{id, response}
    }

    /// Create a new login response error.
    pub fn login_err(id: String, error: api::ErrorDomain) -> Self {
        let mut inner = api::LoginResponse::default();
        inner.error = Some(error);
        let response = Some(Response::Login(inner));
        Self{id, response}
    }

    /// Create a new search giphy response.
    pub fn search_giphy(id: String, gifs: Vec<api::GiphyGif>) -> Self {
        let response = Some(Response::SearchGiphy(api::SearchGiphyResponse{error: None, gifs}));
        Self{id, response}
    }

    /// Create a new search giphy response error.
    pub fn search_giphy_err(id: String, error: api::ErrorDomain) -> Self {
        let mut inner = api::SearchGiphyResponse::default();
        inner.error = Some(error);
        let response = Some(Response::SearchGiphy(inner));
        Self{id, response}
    }
}
