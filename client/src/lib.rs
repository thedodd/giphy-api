#[macro_use]
extern crate seed;

mod json;
mod proto;
mod state;
mod net;
mod view;

use seed::{prelude::*, App};

use crate::{
    state::{update, Model},
    net::open_ws,
    view::view,
};

#[wasm_bindgen]
pub fn start() {
    log!("Booting client application.");
    let app = App::build(Model::default(), update, view).finish().run();
    open_ws(app);
}
