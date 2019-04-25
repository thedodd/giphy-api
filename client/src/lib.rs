#[macro_use]
extern crate seed;

mod api;
mod components;
mod containers;
mod router;
mod state;
mod ui;
mod utils;

use std::rc::Rc;

use common::User;
use console_log;
use gloo::websocket::{
    WsMessage,
    cb::WebSocket,
};
use log::{info, Level};
use seed::{prelude::*, App};

use crate::{
    state::{update, Model, ModelEvent},
    containers::app,
};

/// A type alias for more easily referring to our application state system.
pub type AppState = App<ModelEvent, Model>;

#[wasm_bindgen]
pub fn start() {
    // Initialize logger.
    console_log::init_with_level(Level::Debug).expect("Exepcted logging initialization to succeed.");

    info!("Building initial WebSocket connection.");
    let current_host = match seed::window().location().host() {
        Ok(h) => h,
        Err(err) => {
            panic!("Failed to access window host. {:?}", err);
        }
    };
    let wsres = WebSocket::connect(format!("ws://{}/ws/", current_host))
        .onmessage(|msg: WsMessage| info!("Message received: {:?}", msg))
        .build();
    let ws = match wsres {
        Ok(ws) => Rc::new(ws),
        Err(err) => {
            panic!("Failed to build WebSocket. {:?}", err);
        }
    };

    info!("Building app.");
    let mut model = Model::default();
    model.is_initializing = true;
    let app = App::build(model, update, app)
        .routes(router::router).finish().run();

    info!("Loading session state.");
    match utils::get_session_item("user") {
        Ok(user_json) => match serde_json::from_str::<User>(&user_json) {
            Ok(user) => app.update(state::ModelEvent::Initialized(Some(user), ws)),
            Err(_) => app.update(state::ModelEvent::Initialized(None, ws)),
        }
        Err(_) => app.update(state::ModelEvent::Initialized(None, ws)),
    }
}
