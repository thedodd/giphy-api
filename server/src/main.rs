mod app;
mod config;
mod db;
mod handlers;
mod jwt;
mod models;
mod proto;
mod sockets;

use std::sync::Arc;

use actix::prelude::*;
use env_logger;
use log::info;
use wither::prelude::*;

use crate::{
    app::new_app,
    db::MongoExecutor,
    models::User,
    handlers::SocketHandler,
};

fn main() {
    let cfg = Arc::new(config::Config::new());
    let _ = env_logger::init();

    // Connect to DB backend & sync models.
    let db = MongoExecutor::new(&*cfg).expect("Unable to connect to database backend.");
    info!("Synchronizing data models.");
    User::sync(db.0.clone()).expect("Faild to sync User model.");

    // Boot the various actors of this system.
    let sys = actix::System::new("api");
    let db_executor = SyncArbiter::start(4, move || db.clone());
    let socket_handler = Arbiter::start(move |_| SocketHandler::new(cfg.clone(), db_executor.clone()));
    let _server = new_app(socket_handler);
    let _ = sys.run();
}
