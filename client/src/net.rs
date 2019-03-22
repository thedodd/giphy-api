use prost::Message;
use seed::{prelude::*, App};
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{
    console::{log_1, log_2},
    BinaryType, MessageEvent, WebSocket,
};

use crate::{
    proto::api::{LoginRequest, RequestFrame},
    state::{Model, ModelEvent},
};

const WS_URL: &str = "ws://127.0.0.1:8080/ws/";

pub fn open_ws(state: App<ModelEvent, Model>) {
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

    let s = state.clone();
    let on_message = Closure::wrap(Box::new(move |ev: MessageEvent| {
        log_1(&"Client received a message".into());
        log_2(&"Event type:".into(), &ev.type_().into());
        // let txt = ev.data().as_string().unwrap();
        // let json: json::ServerMsg = serde_json::from_str(&txt).unwrap();
        // log_2(&"text message:".into(), &txt.into());
        // s.update(ModelEvent::ServerMsg(json));
    }) as Box<FnMut(MessageEvent)>);

    let on_error = Closure::wrap(Box::new(|_| {
        log_1(&"err".into());
    }) as Box<FnMut(JsValue)>);

    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();
    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();
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
