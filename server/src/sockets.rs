use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::ws;
use log::{debug};

use crate::{
    app::AppState,
    handlers::{Request, SocketHandler},
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
