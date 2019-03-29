use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use validator::{Validate};
use validator_derive::Validate;

//////////////////////////////////////////////////////////////////////////////////////////////////
// Common Components /////////////////////////////////////////////////////////////////////////////

/// An API response.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag="result", content="payload")]
pub enum Response<D> {
    /// A success payload with data.
    Data(D),

    /// An error payload with an error.
    Error(Error),
}

/// An error coming form the API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Error {
    /// A description of the error.
    pub description: String,

    /// The HTTP status code which represents this type of error.
    pub status: u32,

    /// Error information specific to fields of the domain request.
    pub fields: HashMap<String, String>,
}

impl Error {
    const ISE: &'static str = "Internal server error.";

    /// Create a new instance.
    pub fn new(desc: &str, status: u32, fields: Option<HashMap<String, String>>) -> Self {
        let description = desc.to_string();
        let fields = fields.unwrap_or_default();
        Error{description, status, fields}
    }

    /// Create a new instance representing an internal server error.
    pub fn new_ise() -> Self {
        Self::new(Self::ISE, 500, None)
    }
}

/// A GIF from the Giphy API.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GiphyGif {
    /// The ID of the GIF in Giphy.
    pub id: String,

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
    pub id: String,
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
    #[validate(length(min="6", message="Password must be at least 6 characters in length."))]
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
