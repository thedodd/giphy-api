use actix::prelude::*;
use log::info;

/// A request frame which has come in from a connected socket.
pub struct RequestFrame(pub Vec<u8>);

impl Message for RequestFrame {
    type Result = Vec<u8>;
}

//////////////////////////////////////////////////////////////////////////////////////////////////
// SocketHandler /////////////////////////////////////////////////////////////////////////////////

/// An actor used for handling websocket events.
pub struct SocketHandler;

impl Actor for SocketHandler {
    type Context = Context<Self>;
}

/// Handle binary websocket frames.
impl Handler<RequestFrame> for SocketHandler {
    type Result = MessageResult<RequestFrame>;

    fn handle(&mut self, _msg: RequestFrame, _: &mut Context<Self>) -> Self::Result {
        info!("Handling client request in SocketHandler.");
        MessageResult(Vec::with_capacity(0))
    }
}
