mod error;

#[cfg(feature="server")]
use actix_web::{HttpRequest, HttpResponse, Responder, http::StatusCode};
use serde::{Deserialize, Serialize};
use validator::{Validate};
use validator_derive::Validate;

pub use crate::error::Error;

//////////////////////////////////////////////////////////////////////////////////////////////////
// Common Components /////////////////////////////////////////////////////////////////////////////

/// An API response.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag="result", content="payload")]
pub enum Response<D> {
    /// A success payload with data.
    #[serde(rename="data")]
    Data(D),
    /// An error payload with an error.
    #[serde(rename="error")]
    Error(Error),
}

/// A GIF from the Giphy API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GiphyGif {
    /// The ID of the GIF in Giphy.
    pub id: i64,
    /// The title of the GIF.
    pub title: String,
    /// The URL of the GIF.
    pub url: String,
    /// A bool indicating if the calling user has already saved this GIF.
    pub is_saved: bool,
    /// The optional category for this GIF.
    ///
    /// NB: This does not come from Giphy, this comes from our DB.
    pub category: Option<String>,
}

/// A user of the system.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub jwt: String,
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// Request & Response Models /////////////////////////////////////////////////////////////////////
//
// Message types here come in pairs. If a Request is sent with a particular request variant, its
// corresponding Response variant will be returned. This invariant is part of the API's contract.

/// A request to register a new account.
#[derive(Debug, Deserialize, Serialize, Clone, Validate)]
pub struct RegisterRequest {
    #[validate(email(message="Must provide a valid email address."))]
    pub email: String,
    #[validate(length(min=6, message="Password must be at least 6 characters in length."))]
    pub password: String,
}

/// The response to a register request.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct RegisterResponse(pub User);

/// A login request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// The response to a login request.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(transparent)]
pub struct LoginResponse(pub User);

/// A request to search the Giphy API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchGiphyRequest {
    /// The search string to be used for querying Giphy.
    pub query: String,
}

/// The response to a Giphy search request.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchGiphyResponse {
    /// The GIFs returned from the search.
    pub gifs: Vec<GiphyGif>,
}

/// A reqeust to save a GIF.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveGifRequest {
    // The ID of the Giphy GIF to save.
    pub id: String,
}

/// The response to a request to save a GIF.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SaveGifResponse {
    /// The saved GIF.
    pub gif: GiphyGif,
}

/// A request to fetch the caller's saved GIFs.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetchFavoritesRequest;

/// The response to a request to fetch the caller's saved GIFs.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FetchFavoritesResponse {
    pub gifs: Vec<GiphyGif>,
}

/// A request to categorize a GIF.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CategorizeGifRequest {
    /// The ID of the GIF to update.
    pub id: String,
    /// The new category to apply.
    pub category: String,
}

/// The response to a request to categorize a GIF.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CategorizeGifResponse {
    pub gif: GiphyGif,
}

#[cfg(feature="server")]
impl<T> Responder for Response<T> where T: Serialize {
    type Future = futures::future::Ready<Result<HttpResponse, <Self as Responder>::Error>>;
    type Error = actix_web::Error;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        match &self {
            Self::Data(_) => futures::future::ready(Ok(HttpResponse::Ok().json(self))),
            Self::Error(err) => {
                let status = StatusCode::from_u16(err.status)
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                futures::future::ready(Ok(HttpResponse::build(status).json(self)))
            }
        }
    }
}
