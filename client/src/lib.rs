#[macro_use]
extern crate seed;

mod components;
mod containers;
mod proto;
mod net;
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
    let app = App::build(Model::default(), update, app).finish().run();
    open_ws(app);
}
