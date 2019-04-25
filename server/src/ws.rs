use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use actix::prelude::*;
use actix_web::ws;
use futures::{future::ok};
use log::{debug};
// use prost::Message as ProtoMessage;
use reqwest::r#async::Client;
// use serde_derive::{Deserialize, Serialize};

use crate::{
    app::AppState,
    config::Config,
    db::{MongoExecutor},
};

/// Interval for sending heartbeats to the client.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// Duration before a socket is closed due to inactivity.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// An actor type which represents a live websocket connection.
pub struct SocketState {
    hb: Instant,
    handler: Addr<SocketHandler>,
}

impl Actor for SocketState {
    type Context = ws::WebsocketContext<Self, AppState>;

    /// Method is called on actor start. Handle socket startup routines here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for SocketState {
    /// Handle socket events.
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        match msg {
            ws::Message::Close(_) => {
                debug!("Closing socket.");
                ctx.stop();
            }
            ws::Message::Ping(msg) => {
                debug!("Received a client ping.");
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                debug!("Received a client pong.");
            }
            ws::Message::Text(text) => {
                debug!("Received a text frame: {}", &text);
            }
            ws::Message::Binary(mut frame) => self
                .handler
                .send(Request(frame.take().to_vec()))
                .into_actor(self)
                .then(|res, _actor, ctx| {
                    match res {
                        Ok(inner) => match inner {
                            Ok(payload) => ctx.binary(payload),
                            Err(_) => unreachable!(),
                        }
                        Err(_) => unreachable!(),
                    }
                    fut::ok(())
                })
                .spawn(ctx),
        }
    }
}

impl SocketState {
    /// Create a new instance.
    pub fn new(handler: Addr<SocketHandler>) -> Self {
        Self {
            hb: Instant::now(),
            handler,
        }
    }

    /// Set up a heartbeat with the client.
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Monitor client heartbeats.
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // This connection has timed out.
                debug!("Websocket connection has timed out, closing!");
                ctx.stop();
                return;
            }

            // Client is still alive. Send a heartbeat.
            debug!("Sending client a heartbeat.");
            ctx.ping("");
        });
    }
}

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
    pub fn new(config: Arc<Config>, db: Addr<MongoExecutor>, http: Client) -> Self {
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
        // // Decode received frame.
        // let frame = match RequestFrame::decode(msg.0) {
        //     Ok(frame) => frame,
        //     Err(err) => {
        //         error!("Failed to decode received frame. {:?}", err);
        //         let mut buf = vec![];
        //         let res = ResponseFrame::error(None, api::ErrorResponse::new_invalid());
        //         res.encode(&mut buf).unwrap(); // This will never fail.
        //         return Box::new(ok(buf));
        //     }
        // };
        // debug!("Message received: {:?}", &msg);
        debug!("Message received over WS handler with len: {}", msg.0.len());

        // // Route the message to the appropriate handler.
        // use request_frame::Request::{Register, Login, SearchGiphy};
        // let res_future = match frame.request {
        //     Some(Register(data)) => self.register(frame.id, data),
        //     Some(Login(data)) => self.login(frame.id, data),
        //     Some(SearchGiphy(data)) => self.search_giphy(frame.id, data),
        //     None => {
        //         error!("Unrecognized request variant.");
        //         let error = api::ErrorResponse::new_invalid();
        //         Box::new(futures::future::ok(ResponseFrame::error(Some(&frame.id), error)))
        //     }
        // };

        // // Encode the response to be sent back over the socket.
        // Box::new(res_future.map(|frame| {
        //     let mut buf = vec![];
        //     frame.encode(&mut buf).unwrap(); // This will never fail.
        //     buf
        // }))
        Box::new(ok(vec![]))
    }
}
