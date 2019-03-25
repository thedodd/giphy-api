use std::sync::Arc;

use actix::prelude::*;
use futures::{prelude::*, future::ok};
use log::{debug, error};
use prost::Message as ProtoMessage;
use reqwest::r#async::Client;
use serde_derive::{Deserialize, Serialize};

use crate::{
    config::Config,
    db::{CreateUser, FindUserWithCreds, MongoExecutor},
    jwt::Claims,
    models::User,
    proto::{
        api::{
            self, request_frame,
            RequestFrame, ResponseFrame,
            RegisterRequest, LoginRequest, SearchGiphyRequest,
        },
        api_ext::Error::{self, ErrSystem},
    },
};

/// The API endpoint for querying the Giphy API.
const GIPHY_API_URL: &str = "https://api.giphy.com/v1/gifs/search";

/// A request frame which has come in from a connected socket.
pub struct Request(pub Vec<u8>);

impl Message for Request {
    type Result = Result<Vec<u8>, ()>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// SocketHandler /////////////////////////////////////////////////////////////////////////////////

/// An actor used for handling websocket events.
pub struct SocketHandler {
    config: Arc<Config>,
    db: Addr<MongoExecutor>,
    http: Client,
}

impl SocketHandler {
    /// Create a new instance.
    pub fn new(config: Arc<Config>, db: Addr<MongoExecutor>) -> Self {
        let http = Client::new();
        Self{config, db, http}
    }
}

impl Actor for SocketHandler {
    type Context = Context<Self>;
}

/// Handle binary websocket frames.
impl Handler<Request> for SocketHandler {
    type Result = ResponseFuture<Vec<u8>, ()>;

    fn handle(&mut self, msg: Request, _: &mut Context<Self>) -> Self::Result {
        // Decode received frame.
        let frame = match RequestFrame::decode(msg.0) {
            Ok(frame) => frame,
            Err(err) => {
                error!("Failed to decode received frame. {:?}", err);
                let mut buf = vec![];
                let res = ResponseFrame::error(None, api::ErrorResponse::new_invalid());
                res.encode(&mut buf).unwrap(); // This will never fail.
                return Box::new(ok(buf));
            }
        };
        debug!("Message received: {:?}", &frame);

        // Route the message to the appropriate handler.
        use request_frame::Request::{Register, Login, SearchGiphy};
        let res_future = match frame.request {
            Some(Register(data)) => self.register(frame.id, data),
            Some(Login(data)) => self.login(frame.id, data),
            Some(SearchGiphy(data)) => self.search_giphy(frame.id, data),
            None => {
                error!("Unrecognized request variant.");
                let error = api::ErrorResponse::new_invalid();
                Box::new(futures::future::ok(ResponseFrame::error(Some(&frame.id), error)))
            }
        };

        // Encode the response to be sent back over the socket.
        Box::new(res_future.map(|frame| {
            let mut buf = vec![];
            frame.encode(&mut buf).unwrap(); // This will never fail.
            buf
        }))
    }
}

impl SocketHandler {
    /// Handle registration requests.
    fn register(&self, rqid: String, data: RegisterRequest) -> Box<dyn Future<Item=ResponseFrame, Error=()>> {
        // Register the new user.
        let rqid_copy = rqid.clone();
        let cfg = self.config.clone();
        let f = self.db.send(CreateUser{email: data.email, password: data.password})
            .then(|res| match res {
                Ok(inner) => inner,
                Err(mailbox_err) => {
                    error!("Actix mailbox error. {:?}", mailbox_err);
                    Err(ErrSystem(api::ErrorResponse::new_ise()))
                }
            })
            .and_then(move |user: User| {
                // Generate JWT for user & build response.
                let user_id = user.id.map(|id| id.to_hex()).unwrap_or_default();
                let jwt = Claims::new(&cfg.raw_idp_private_key, user_id.clone()).map_err(ErrSystem)?;
                Ok(ResponseFrame::register(rqid, user_id, user.email, jwt))
            })
            .then(move |res| match res {
                Ok(ok) => Ok(ok),
                Err(err) => match err {
                    Error::ErrSystem(e) => Ok(ResponseFrame::error(Some(&rqid_copy), e)),
                    Error::ErrDomain(e) => Ok(ResponseFrame::register_err(rqid_copy, e)),
                }
            });

        Box::new(f)
    }

    /// Handle login requests.
    fn login(&self, rqid: String, data: LoginRequest) -> Box<dyn Future<Item=ResponseFrame, Error=()>> {
        // Check the provided credentials and log the user in.
        let rqid_copy = rqid.clone();
        let cfg = self.config.clone();
        let f = self.db.send(FindUserWithCreds{email: data.email, password: data.password})
            .then(|res| match res {
                Ok(inner) => inner,
                Err(mailbox_err) => {
                    error!("Actix mailbox error. {:?}", mailbox_err);
                    Err(ErrSystem(api::ErrorResponse::new_ise()))
                }
            })
            .and_then(move |user: User| {
                // Generate JWT for user & build response.
                let user_id = user.id.map(|id| id.to_hex()).unwrap_or_default();
                let jwt = Claims::new(&cfg.raw_idp_private_key, user_id.clone()).map_err(ErrSystem)?;
                Ok(ResponseFrame::login(rqid, user_id, user.email, jwt))
            })
            .then(move |res| match res {
                Ok(ok) => Ok(ok),
                Err(err) => match err {
                    Error::ErrSystem(e) => Ok(ResponseFrame::error(Some(&rqid_copy), e)),
                    Error::ErrDomain(e) => Ok(ResponseFrame::login_err(rqid_copy, e)),
                }
            });

        Box::new(f)
    }

    /// Handle search giphy requests.
    fn search_giphy(&self, rqid: String, data: SearchGiphyRequest) -> Box<dyn Future<Item=ResponseFrame, Error=()>> {
        // FUTURE: in order to provide maximum security over the stateless JWT auth protocol,
        // we can introduce a nonce value to the user model. If the JWT's nonce (a timestamp)
        // does not match the user model, then the JWT is invalid.

        // Validate the given JWT before processing request.
        let _claims = match Claims::from_jwt(&data.jwt, &self.config.raw_idp_public_key) {
            Ok(claims) => claims,
            Err(err) => return Box::new(ok(ResponseFrame::error(Some(&rqid), err))),
        };

        // TODO: join this with another future which queries the DB for favorites.

        // Fetch a payload of Gifs from Giphy according to the given search.
        let (rqid_copy0, rqid_copy1) = (rqid.clone(), rqid.clone());
        let query_fut = self.http.get(GIPHY_API_URL)
            .query(&[("api_key", self.config.giphy_api_key.as_str()), ("q", data.query.as_str()), ("limit", "50")])

            // Send the request & do initial error handling.
            .send()
            .and_then(|res| res.error_for_status())
            .and_then(|mut result| {
                result.json::<GiphySearchResponse>()
            })
            .then(move |res| match res {
                Ok(payload) => {
                    let gifs = payload.data.into_iter().map(|gif| {
                        api::GiphyGif{
                            id: gif.id,
                            title: gif.title,
                            url: gif.images.downsized_medium.url,
                            is_favorite: false,
                        }
                    }).collect();
                    Ok(ResponseFrame::search_giphy(rqid_copy0, gifs))
                },
                Err(err) => {
                    error!("Error from query to the Giphy API. {:?}", err);
                    Err(api::ErrorResponse::new_ise())
                }
            })
            .then(move |res| match res {
                Ok(ok) => Ok(ok),
                Err(err) => Ok(ResponseFrame::error(Some(&rqid_copy1), err)),
            });

        Box::new(query_fut)
    }
}

#[derive(Deserialize, Serialize)]
struct GiphySearchResponse {
    pub data: Vec<GiphySearchGif>,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGif {
    pub id: String,
    pub title: String,
    pub images: GiphySearchGifImages,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGifImages {
    pub downsized_medium: GiphySearchGifImagesModel,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGifImagesModel {
    #[serde(alias="mp4")]
    pub url: String,
}
