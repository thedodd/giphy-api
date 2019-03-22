use actix::prelude::*;
use log::info;
use prost::Message as ProtoMessage;

use crate::proto::api::{
    RequestFrame, ResponseFrame, LoginResponse,
};

/// A request frame which has come in from a connected socket.
pub struct Request(pub Vec<u8>);

impl Message for Request {
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
impl Handler<Request> for SocketHandler {
    type Result = MessageResult<Request>;

    fn handle(&mut self, msg: Request, _: &mut Context<Self>) -> Self::Result {
        info!("Handling client request in SocketHandler.");
        let frame = RequestFrame::decode(msg.0).expect("Expected to be able to decode received frame.");
        info!("Message received: {:?}", &frame);

        let mut buf = vec![];
        let res = ResponseFrame::login(frame.id.clone(), LoginResponse{error: None, jwt: frame.id});
        res.encode(&mut buf).unwrap(); // This will never fail.
        MessageResult(buf)
    }
}
