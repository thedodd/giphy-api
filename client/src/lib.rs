mod api;
mod components;
mod containers;
mod router;
mod state;
mod ui;
mod utils;

use common::User;
use seed::{prelude::*, App};

use crate::containers::app;
use crate::state::{update, Model, ModelEvent};

#[wasm_bindgen]
pub fn start() {
    seed::log!("Booting client application.");
    let mut model = Model::default();
    model.is_initializing = true;
    let app = App::builder(update, app)
        .routes(router::router)
        .build_and_start();

    // Attempt to load any session data.
    match utils::get_session_item("user") {
        Ok(user_json) => match serde_json::from_str::<User>(&user_json) {
            Ok(user) => app.update(state::ModelEvent::Initialized(Some(user))),
            Err(_) => app.update(state::ModelEvent::Initialized(None)),
        }
        Err(_) => app.update(state::ModelEvent::Initialized(None)),
    }
}
