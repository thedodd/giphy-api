//////////////////////////////////////////////////////////////////////////////////////////////////
/// Components ///////////////////////////////////////////////////////////////////////////////////

/// A user friendly description of an error which has taken place on the server.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Error {
    /// A user friendly message describing the error.
    #[prost(string, tag="1")]
    pub description: std::string::String,
    /// The HTTP status which classifies this type of error.
    #[prost(uint32, tag="2")]
    pub status: u32,
    /// An internal code used to drive client behavior.
    #[prost(string, tag="3")]
    pub code: std::string::String,
    /// Additional metadata on the error.
    #[prost(map="string, string", tag="4")]
    pub meta: ::std::collections::HashMap<std::string::String, std::string::String>,
}
//////////////////////////////////////////////////////////////////////////////////////////////////
/// Request & Response Variants //////////////////////////////////////////////////////////////////
///
/// Message types here come in pairs. If a Request is sent with a particular request variant, its
/// corresponding Response variant will be returned. This invariant is part of the API's contract.

/// A login request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginRequest {
    #[prost(string, tag="1")]
    pub username: std::string::String,
    #[prost(string, tag="2")]
    pub password: std::string::String,
}
/// The response to a login request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LoginResponse {
    #[prost(message, optional, tag="1")]
    pub error: ::std::option::Option<Error>,
    #[prost(string, tag="2")]
    pub jwt: std::string::String,
}
/// A request to refresh the given JWT.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefreshTokenRequest {
    #[prost(string, tag="3")]
    pub jwt: std::string::String,
}
/// The response to a refresh token request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefreshTokenResponse {
    #[prost(message, optional, tag="1")]
    pub error: ::std::option::Option<Error>,
    #[prost(string, tag="2")]
    pub jwt: std::string::String,
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
    #[prost(oneof="request_frame::Request", tags="10, 11")]
    pub request: ::std::option::Option<request_frame::Request>,
}
pub mod request_frame {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Request {
        #[prost(message, tag="10")]
        Login(super::LoginRequest),
        #[prost(message, tag="11")]
        RefreshToken(super::RefreshTokenRequest),
    }
}
/// A data frame which represents an API response.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseFrame {
    /// The ID of the original request which produced this frame.
    #[prost(string, tag="1")]
    pub id: std::string::String,
    #[prost(oneof="response_frame::Response", tags="10, 11")]
    pub response: ::std::option::Option<response_frame::Response>,
}
pub mod response_frame {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag="10")]
        Login(super::LoginResponse),
        #[prost(message, tag="11")]
        RefreshToken(super::RefreshTokenResponse),
    }
}
