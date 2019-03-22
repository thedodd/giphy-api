mod handlers;
mod proto;
mod sockets;

use std::path::Path;

use actix::prelude::*;
use actix_web::{
    server, ws,
    fs::{NamedFile, StaticFileConfig, StaticFiles},
    App, HttpRequest, HttpResponse, Result,
};
use env_logger;
use log::info;

use crate::{handlers::SocketHandler, sockets::SocketState};

const STATIC_DIR: &str = "./static";

#[derive(Default)]
struct StaticFileServerConfig;

impl StaticFileConfig for StaticFileServerConfig {
    fn is_use_etag() -> bool {
        true
    }

    fn is_use_last_modifier() -> bool {
        true
    }
}

/// The application state object.
pub struct AppState {
    socket_handler: Addr<SocketHandler>,
}

/// A simple handler for setting up WS connections.
fn handle_socket(req: &HttpRequest<AppState>) -> Result<HttpResponse> {
    let socket_handler = req.state().socket_handler.clone();
    ws::start(req, SocketState::new(socket_handler))
}

fn main() {
    let _ = env_logger::init();
    let sys = actix::System::new("api");

    // Start the socket handler actor.
    let socket_handler = Arbiter::start(|_| SocketHandler);

    server::new(move || {
        let state = AppState {
            socket_handler: socket_handler.clone(),
        };

        App::with_state(state)
            .resource("/ws/", |r| r.route().f(handle_socket))

            // Build static file handler.
            .handler("/", StaticFiles::with_config(STATIC_DIR, StaticFileServerConfig)
                .expect("Failed to build static file handler. Probably means static dir is bad or there are FS issues.")
                .default_handler(|_req: &HttpRequest<AppState>| -> Result<NamedFile> {
                    let path = Path::new(STATIC_DIR).join("index.html");
                    Ok(NamedFile::open(path)?)
                }))

            .finish()
    })
    .bind("127.0.0.1:8080") // TODO: 0.0.0.0
    .expect("Expected to bind 127.0.0.1:8080 successfully.")
    .start();

    info!("Server is listening on 127.0.0.1:8080.");
    let _ = sys.run();
}
