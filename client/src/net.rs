use js_sys::{ArrayBuffer, Function, Uint8Array};
use prost::Message;
use seed::prelude::*;
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{
    console::{log_1, log_2},
    BinaryType, MessageEvent, WebSocket,
};

use crate::{
    AppState,
    proto::api::{LoginRequest, RequestFrame, ResponseFrame},
    state::{ModelEvent},
};

const WS_URL: &str = "ws://127.0.0.1:8080/ws/";

pub fn open_ws(state: AppState) {
    let ws = WebSocket::new(WS_URL).unwrap();
    ws.set_binary_type(BinaryType::Arraybuffer);

    let s = state.clone();
    let on_open = Closure::wrap(Box::new(move |_| {
        log!("WebSocket connection is open now");
        s.update(ModelEvent::Connected);
    }) as Box<FnMut(JsValue)>);

    let on_close = Closure::wrap(Box::new(|_| {
        log!("WebSocket connection was closed");
    }) as Box<FnMut(JsValue)>);

    // Build message handler.
    let on_message = build_message_handler(state.clone());
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    // Build error handler.
    let on_error = Closure::wrap(Box::new(|_| {
        log_1(&"err".into());
    }) as Box<FnMut(JsValue)>);

    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    // Register a handler for sending messages to server on specific types of state changes.
    // TODO: use a different pattern for this.
    let app = state.clone();
    state.add_message_listener(move |msg| match msg {
        ModelEvent::Send(msg) => {
            let request = RequestFrame::login(LoginRequest{
                username: String::from("user"),
                password: String::from("pass"),
            });
            let mut buf = vec![];
            request.encode(&mut buf).unwrap(); // This will never fail.
            ws.send_with_u8_array(buf.as_mut_slice()).expect("Expected to be able to send socket message.");
            app.update(ModelEvent::Sent);

            // let s = serde_json::to_string(msg).unwrap();
            // ws.send_with_str(&s).unwrap();
            // app.update(ModelEvent::Sent);
        }
        _ => {}
    });
}

/// The handler function used for websocket connections.
fn build_message_handler(state: AppState) -> Closure<(dyn FnMut(MessageEvent) + 'static)> {
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
        log!(format!("Decoded message: {:?}", &frame));
        state.update(ModelEvent::ServerMsg(frame));
    };
    Closure::wrap(Box::new(handler) as Box<FnMut(MessageEvent)>)
}
