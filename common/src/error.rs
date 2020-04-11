use std::collections::HashMap;

#[cfg(feature="server")]
use actix_web::{dev::Body, error::ResponseError, web::HttpResponse, http::StatusCode};
use serde::{Deserialize, Serialize};

/// An error coming form the API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Error {
    /// A description of the error.
    pub description: String,

    /// The HTTP status code which represents this type of error.
    pub status: u16,

    /// Error information specific to fields of the domain request.
    pub fields: HashMap<String, String>,
}

impl Error {
    const ISE: &'static str = "Internal server error.";

    /// Create a new instance.
    pub fn new(desc: &str, status: u16, fields: Option<HashMap<String, String>>) -> Self {
        let description = desc.to_string();
        let fields = fields.unwrap_or_default();
        Error{description, status, fields}
    }

    /// Create a new instance representing an internal server error.
    pub fn new_ise() -> Self {
        Self::new(Self::ISE, 500, None)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// Server Impl ///////////////////////////////////////////////////////////////////////////////////

#[cfg(feature="server")]
impl Error {
    const BODY_MALFORMED: &'static str = "Malformed body in request.";
    const INVALID_CREDENTIALS: &'static str = "Invalid credentials provided.";
    const UNAUTHORIZED_TOKEN_EXPIRED: &'static str = "Unauthorized. Token is expired.";
    const UNAUTHORIZED_INVALID_TOKEN: &'static str = "Unauthorized. Invalid token provided.";

    /// Create a new error indicating that the body of the request was malformed.
    pub fn new_deser_err(src: serde_json::error::Error) -> Self {
        tracing::error!("{}", src);
        Self::new(&format!("{} {}", Self::BODY_MALFORMED, src), 400, None)
    }

    /// Create a new error indicating that the given credentials were invalid.
    pub fn new_invalid_credentials() -> Self {
        Self::new(Self::INVALID_CREDENTIALS.into(), 401, None)
    }

    /// Create a new error indicating that the given JWT was expired.
    pub fn new_token_expired() -> Self {
        Self::new(Self::UNAUTHORIZED_TOKEN_EXPIRED.into(), 401, None)
    }

    /// Create a new error indicating that the given JWT was expired.
    pub fn new_invalid_token() -> Self {
        Self::new(Self::UNAUTHORIZED_INVALID_TOKEN.into(), 401, None)
    }
}

#[cfg(feature="server")]
impl From<validator::ValidationErrors> for Error {
    fn from(src: validator::ValidationErrors) -> Self {
        let mut fields = HashMap::new();
        for (k, v) in src.field_errors() {
            fields.insert(k.into(), v[0].to_string());
        }
        Error::new("Invalid input.", 400, Some(fields))
    }
}

#[cfg(feature="server")]
impl From<sqlx::Error> for Error {
    fn from(src: sqlx::Error) -> Self {
        tracing::error!("{}", src);
        Error::new_ise()
    }
}

#[cfg(feature="server")]
impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
    fn error_response(&self) -> HttpResponse<Body> {
        HttpResponse::build(self.status_code())
            .json2(&crate::Response::<()>::Error(self.clone()))
    }
}
