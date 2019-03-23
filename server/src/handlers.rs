use std::sync::Arc;

use actix::prelude::*;
use futures::prelude::*;
use log::{error, info};
use prost::Message as ProtoMessage;

use crate::{
    config::Config,
    db::{CreateUser, FindUserWithCreds, MongoExecutor},
    jwt::Claims,
    models::User,
    proto::api::{
        self, request_frame,
        RequestFrame, ResponseFrame,
        RegisterRequest, LoginRequest,
    },
};

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
}

impl SocketHandler {
    /// Create a new instance.
    pub fn new(config: Arc<Config>, db: Addr<MongoExecutor>) -> Self {
        Self{config, db}
    }
}

impl Actor for SocketHandler {
    type Context = Context<Self>;
}

/// Handle binary websocket frames.
impl Handler<Request> for SocketHandler {
    type Result = ResponseFuture<Vec<u8>, ()>;

    fn handle(&mut self, msg: Request, _: &mut Context<Self>) -> Self::Result {
        info!("Handling client request in SocketHandler.");
        let frame = RequestFrame::decode(msg.0).expect("Expected to be able to decode received frame.");
        info!("Message received: {:?}", &frame);

        // Route the message to the appropriate handler.
        use request_frame::Request::{Register, Login};
        let res_future = match frame.request {
            Some(Register(data)) => self.register(frame.id, data),
            Some(Login(data)) => self.login(frame.id, data),
            None => {
                error!("Unrecognized request variant.");
                let error = api::Error::new("Unrecognized request variant.", 400, None, None);
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
                    Err(api::Error::new_ise(None, None))
                }
            })
            .and_then(move |user: User| {
                // Generate JWT for user & build response.
                let user_id = user.id.map(|id| id.to_hex()).unwrap_or_default();
                let jwt = Claims::new(&cfg.raw_idp_private_key, user_id.clone())?;
                Ok(ResponseFrame::register(rqid, user_id, user.email, jwt))
            })
            .then(move |res| match res {
                Ok(ok) => Ok(ok),
                Err(err) => Ok(ResponseFrame::error(Some(&rqid_copy), err)),
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
                    Err(api::Error::new_ise(None, None))
                }
            })
            .and_then(move |user: User| {
                // Generate JWT for user & build response.
                let user_id = user.id.map(|id| id.to_hex()).unwrap_or_default();
                let jwt = Claims::new(&cfg.raw_idp_private_key, user_id.clone())?;
                Ok(ResponseFrame::login(rqid, user_id, user.email, jwt))
            })
            .then(move |res| match res {
                Ok(ok) => Ok(ok),
                Err(err) => Ok(ResponseFrame::error(Some(&rqid_copy), err)),
            });

        Box::new(f)
    }
}
