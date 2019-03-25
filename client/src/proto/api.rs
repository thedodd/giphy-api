//////////////////////////////////////////////////////////////////////////////////////////////////
/// Components ///////////////////////////////////////////////////////////////////////////////////

/// A type representing an error which has taken place in some domain specific manner.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ErrorDomain {
    /// A description of the error.
    #[prost(string, tag="1")]
    pub description: std::string::String,
    /// Error information specific to fields of the domain request.
    #[prost(map="string, string", tag="2")]
    pub fields: ::std::collections::HashMap<std::string::String, std::string::String>,
}
/// A GIF from the Giphy API.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GiphyGif {
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(string, tag="2")]
    pub title: std::string::String,
    #[prost(string, tag="3")]
    pub url: std::string::String,
    #[prost(bool, tag="4")]
    pub is_favorite: bool,
}
/// A GIF which has been favorited by a user.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FavoriteGif {
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(message, optional, tag="2")]
    pub gif: ::std::option::Option<GiphyGif>,
    #[prost(string, tag="3")]
    pub category: std::string::String,
}
//////////////////////////////////////////////////////////////////////////////////////////////////
/// Request & Response Variants //////////////////////////////////////////////////////////////////
///
/// Message types here come in pairs. If a Request is sent with a particular request variant, its
/// corresponding Response variant will be returned. This invariant is part of the API's contract.

/// A response which represents an error outside of the corresponding request's domain.
///
/// This typically will be returned as part of an authentication failure, or an ISE.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ErrorResponse {
    /// A user friendly message describing the error.
    #[prost(string, tag="1")]
    pub description: std::string::String,
    /// The type of error which this represents.
    #[prost(enumeration="ErrorResponseType", tag="2")]
    pub etype: i32,
}
/// A request to register a new account.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterRequest {
    #[prost(string, tag="1")]
    pub email: std::string::String,
    #[prost(string, tag="2")]
    pub password: std::string::String,
}
/// The response to a register request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterResponse {
    #[prost(message, optional, tag="1")]
    pub error: ::std::option::Option<ErrorDomain>,
    /// The user's ID.
    #[prost(string, tag="2")]
    pub id: std::string::String,
    #[prost(string, tag="3")]
    pub email: std::string::String,
    #[prost(string, tag="4")]
    pub jwt: std::string::String,
}
/// A login request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginRequest {
    #[prost(string, tag="1")]
    pub email: std::string::String,
    #[prost(string, tag="2")]
    pub password: std::string::String,
}
/// The response to a login request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginResponse {
    #[prost(message, optional, tag="1")]
    pub error: ::std::option::Option<ErrorDomain>,
    /// The user's ID.
    #[prost(string, tag="2")]
    pub id: std::string::String,
    #[prost(string, tag="3")]
    pub email: std::string::String,
    #[prost(string, tag="4")]
    pub jwt: std::string::String,
}
/// A request to search the Giphy API.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchGiphyRequest {
    #[prost(string, tag="1")]
    pub jwt: std::string::String,
    #[prost(string, tag="2")]
    pub query: std::string::String,
}
/// The response to a Giphy search request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchGiphyResponse {
    #[prost(message, optional, tag="1")]
    pub error: ::std::option::Option<ErrorDomain>,
    #[prost(message, repeated, tag="2")]
    pub gifs: ::std::vec::Vec<GiphyGif>,
}
//////////////////////////////////////////////////////////////////////////////////////////////////
/// Root Frame ///////////////////////////////////////////////////////////////////////////////////
///
/// The contract of this API is that any message sent to this API must be a RequestFrame.
/// Similarly, the API will only ever send ResponseFrames over a connected websocket.

/// A data frame which represents an API request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestFrame {
    /// The ID of this request.
    ///
    /// Clients must ensure that this ID is unique, as it is the primary mechanism used for matching
    /// request frames with response frames over a websocket.
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(oneof="request_frame::Request", tags="10, 11, 12")]
    pub request: ::std::option::Option<request_frame::Request>,
}
pub mod request_frame {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Request {
        #[prost(message, tag="10")]
        Register(super::RegisterRequest),
        #[prost(message, tag="11")]
        Login(super::LoginRequest),
        #[prost(message, tag="12")]
        SearchGiphy(super::SearchGiphyRequest),
    }
}
/// A data frame which represents an API response.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFrame {
    /// The ID of the original request which produced this frame.
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(oneof="response_frame::Response", tags="10, 11, 12, 13")]
    pub response: ::std::option::Option<response_frame::Response>,
}
pub mod response_frame {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag="10")]
        Error(super::ErrorResponse),
        #[prost(message, tag="11")]
        Register(super::RegisterResponse),
        #[prost(message, tag="12")]
        Login(super::LoginResponse),
        #[prost(message, tag="13")]
        SearchGiphy(super::SearchGiphyResponse),
    }
}
/// The possible variants of an error response.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ErrorResponseType {
    /// An internal server error.
    EtypeIse = 0,
    /// An error indicating the that user has failed authentication.
    ///
    /// Typically this means that the user's JWT is expired.
    EtypeAuthn = 1,
    /// An error indicating that the received payload was invalid.
    ///
    /// Typically this means that the frame couldn't even be decoded. Errors specific to business
    /// logic requests MUST NOT appear here.
    EtypeInvalid = 2,
}
