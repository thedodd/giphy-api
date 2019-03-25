use std::rc::Rc;

use js_sys::{ArrayBuffer, Uint8Array};
use prost::Message;
use seed::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{BinaryType, MessageEvent, WebSocket};

use crate::{
    AppState,
    containers::{LoginContainerEvent, SearchContainerEvent},
    proto::api::{ErrorResponseType, RequestFrame, ResponseFrame, response_frame::Response},
    state::{Model, ModelEvent},
};

const WS_URL: &str = "ws://127.0.0.1:8080/ws/";

/// A closure taking a message event.
pub type HandleMessage = Closure<(dyn FnMut(MessageEvent) + 'static)>;

/// A closure taking a JS value.
pub type HandleValue = Closure<(dyn FnMut(JsValue) + 'static)>;

/// An enumeration of the types of closures used here.
pub enum WSClosure {
    HandleM(HandleMessage),
    HandleV(HandleValue),
}

/// The subset of the app's data model related to networking.
#[derive(Clone, Default)]
pub struct NetworkState {
    pub connected: bool,
    pub socket: Option<WebSocket>, // A populated value here does not indicate a live connection.
    pub closures: Vec<Rc<WSClosure>>,
}

/// An enumeration of all network related events to be handled.
#[derive(Clone)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
    // Reconnect, // TODO: implement this. Will need to create a cycle with Box<AppState>.
    NewSocket(WebSocket),
    NewClosure(Rc<WSClosure>),
    SendRequest(RequestFrame),
    Received(ResponseFrame),
}

impl NetworkEvent {
    /// The reducer for this state model.
    pub fn reducer(event: NetworkEvent, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            NetworkEvent::Connected => {
                model.network.connected = true;
                Render.into()
            }
            NetworkEvent::Disconnected => {
                model.network.connected = false;
                model.network.socket = None;
                model.network.closures.clear();
                Render.into()
            }
            // NetworkEvent::Reconnect => {
            //     model.network.connected = false;
            //     model.network.socket = None;
            //     model.network.closures.clear();
            //     // open_ws(state: AppState)
            //     Update::with_msg(effect_msg: Ms)
            //     Render.into()
            // }
            NetworkEvent::NewSocket(ws) => {
                model.network.socket = Some(ws);
                Render.into()
            }
            NetworkEvent::NewClosure(cb) => {
                model.network.closures.push(cb);
                Skip.into()
            }
            NetworkEvent::SendRequest(req) => {
                let ws = match model.network.socket.as_ref() {
                    Some(ws) => ws,
                    None => return Skip.into()
                };
                let mut buf = vec![];
                req.encode(&mut buf).unwrap(); // This will never fail.
                if let Err(err) = ws.send_with_u8_array(buf.as_mut_slice()) {
                    log!(format!("Failed to send request to API. {:?}", err));
                }
                Skip.into()
            }
            NetworkEvent::Received(frame) => {
                // We've received a valid response frame from the server. Route it.
                use ErrorResponseType::{EtypeAuthn, EtypeIse, EtypeInvalid};
                match frame.response {
                    Some(Response::Error(err)) => match ErrorResponseType::from_i32(err.etype) {
                        Some(EtypeAuthn) => Update::with_msg(ModelEvent::Logout),
                        Some(EtypeIse) => {
                            log!("Server returned an ISE. Long term, we would map this to a request ID for handling.");
                            Skip.into()
                        },
                        Some(EtypeInvalid) => {
                            log!("Server received a malformed frame from this client.");
                            Skip.into()
                        },
                        None => {
                            log!("Server returned an unknown error type.");
                            Skip.into()
                        }
                    }
                    Some(Response::Login(res)) =>
                        Update::with_msg(ModelEvent::Login(LoginContainerEvent::LoginResponse(res))),
                    Some(Response::Register(res)) =>
                        Update::with_msg(ModelEvent::Login(LoginContainerEvent::RegisterResponse(res))),
                    Some(Response::SearchGiphy(res)) =>
                        Update::with_msg(ModelEvent::Search(SearchContainerEvent::SearchResponse(res))),
                    None => {
                        log!("Response frame from API did not have an error and did not have a response variant.");
                        Skip.into()
                    }
                }
            }
        }
    }

    /// Emit a new event for adding a network closure.
    pub fn new_closure(state: AppState, cb: WSClosure) {
        state.update(ModelEvent::Network(NetworkEvent::NewClosure(Rc::new(cb))));
    }
}

pub fn open_ws(state: AppState) {
    let ws = WebSocket::new(WS_URL).unwrap(); // TODO: handle this.
    ws.set_binary_type(BinaryType::Arraybuffer);
    state.update(ModelEvent::Network(NetworkEvent::NewSocket(ws.clone())));

    // Build handler for when connections are first open.
    let on_open = build_on_open(state.clone());
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    NetworkEvent::new_closure(state.clone(), WSClosure::HandleV(on_open));

    // Build handler for when connections are closed.
    let on_close = build_on_close(state.clone());
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    NetworkEvent::new_closure(state.clone(), WSClosure::HandleV(on_close));

    // Build message handler.
    let on_message = build_on_message(state.clone());
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    NetworkEvent::new_closure(state.clone(), WSClosure::HandleM(on_message));

    // Build error handler.
    let on_error = build_on_close(state.clone());
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    NetworkEvent::new_closure(state.clone(), WSClosure::HandleV(on_error));
}

/// Generate a handler function for when a connection is open.
fn build_on_open(state: AppState) -> HandleValue {
    let handler = move |_| {
        state.update(ModelEvent::Network(NetworkEvent::Connected));
    };
    Closure::wrap(Box::new(handler) as Box<FnMut(JsValue)>)
}

/// Generate a handler function for when a connection is closed.
fn build_on_close(state: AppState) -> HandleValue {
    let handler = move |_| {
        state.update(ModelEvent::Network(NetworkEvent::Disconnected));
    };
    Closure::wrap(Box::new(handler) as Box<FnMut(JsValue)>)
}

/// Generate a handler function used for websocket connections.
fn build_on_message(state: AppState) -> HandleMessage {
    let handler = move |ev: MessageEvent| {
        // Extract the raw bytes of the message.
        let buf = match ev.data().dyn_into::<ArrayBuffer>() {
            Ok(buf) => {
                let u8buf = Uint8Array::new(&buf);
                let mut decode_buf = vec![0; u8buf.byte_length() as usize];
                u8buf.copy_to(&mut decode_buf);
                decode_buf
            }
            Err(_) => {
                log!("Received an unexpected message from the server which was not a raw byte array.");
                return;
            }
        };

        // Decode the received message to our expected protobuf message type.
        let frame = match ResponseFrame::decode(buf) {
            Ok(frame) => frame,
            Err(err) => {
                log!(format!("Failed to decode server message: {:?}", err));
                return;
            }
        };

        // Process the recived message in our state update system.
        state.update(ModelEvent::Network(NetworkEvent::Received(frame)));
    };
    Closure::wrap(Box::new(handler) as Box<FnMut(MessageEvent)>)
}
