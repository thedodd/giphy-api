#[macro_use]
extern crate seed;

mod proto;
mod state;
mod net;
mod view;

use seed::{prelude::*, App};

use crate::{
    state::{update, Model, ModelEvent},
    net::open_ws,
    view::view,
};

/// A type alias for more easily referring to our application state system.
pub type AppState = App<ModelEvent, Model>;

#[wasm_bindgen]
pub fn start() {
    log!("Booting client application.");
    let app = App::build(Model::default(), update, view).finish().run();
    open_ws(app);
}
