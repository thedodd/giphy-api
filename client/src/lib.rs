#[macro_use]
extern crate seed;

mod components;
mod containers;
mod proto;
mod net;
mod router;
mod state;

use seed::{prelude::*, App};

use crate::{
    state::{update, Model, ModelEvent},
    net::open_ws,
    containers::app,
};

/// A type alias for more easily referring to our application state system.
pub type AppState = App<ModelEvent, Model>;

#[wasm_bindgen]
pub fn start() {
    log!("Booting client application.");
    let mut model = Model::default();
    model.is_initializing = true;
    let app = App::build(model, update, app)
        .routes(router::router).finish().run();

    // Attempt to open initial connection with backend.
    open_ws(app.clone());

    // Attempt to load any session data.
    match state::get_session_item("user") {
        Ok(user_json) => match serde_json::from_str::<state::User>(&user_json) {
            Ok(user) => app.update(state::ModelEvent::Initialized(Some(user))),
            Err(_) => app.update(state::ModelEvent::Initialized(None)),
        }
        Err(_) => app.update(state::ModelEvent::Initialized(None)),
    }
}
