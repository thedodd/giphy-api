#[macro_use]
extern crate seed;

mod api;
mod components;
mod containers;
mod router;
mod state;
mod ui;
mod utils;

use common::User;
use seed::{prelude::*, App};

use crate::{
    state::{update, Model, ModelEvent},
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

    // Attempt to load any session data.
    match utils::get_session_item("user") {
        Ok(user_json) => match serde_json::from_str::<User>(&user_json) {
            Ok(user) => app.update(state::ModelEvent::Initialized(Some(user))),
            Err(_) => app.update(state::ModelEvent::Initialized(None)),
        }
        Err(_) => app.update(state::ModelEvent::Initialized(None)),
    }
}
