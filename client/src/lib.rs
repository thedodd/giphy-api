#[macro_use]
extern crate seed;

mod json;
mod proto;
mod state;
mod view;

use seed::{prelude::*, App};
use wasm_bindgen::JsCast;
use web_sys::{
    console::{log_1, log_2},
    BinaryType, MessageEvent, WebSocket,
};

use crate::{
    state::{update, Model, ModelEvent},
    view::view,
};

const WS_URL: &str = "ws://127.0.0.1:8080/ws/";

fn open_ws(state: App<ModelEvent, Model>) {
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
        let txt = ev.data().as_string().unwrap();
        let json: json::ServerMsg = serde_json::from_str(&txt).unwrap();
        log_2(&"text message:".into(), &txt.into());
        s.update(ModelEvent::ServerMsg(json));
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
            let s = serde_json::to_string(msg).unwrap();
            ws.send_with_str(&s).unwrap();
            app.update(ModelEvent::Sent);
        }
        _ => {}
    });
}

#[wasm_bindgen]
pub fn start() {
    log!("Booting client application.");
    let app = App::build(Model::default(), update, view).finish().run();
    open_ws(app);
}
