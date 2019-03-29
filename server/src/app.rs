use std::{
    path::Path,
    sync::Arc,
};

use actix::prelude::*;
use actix_web::{
    server,
    fs::{NamedFile, StaticFileConfig, StaticFiles},
    http::Method,
    App, HttpRequest, Result,
};
use actix_net::server::Server;
use log::info;
use reqwest::r#async::Client;

use crate::{
    config::Config,
    db::MongoExecutor,
    handlers,
};

const STATIC_DIR: &str = "./static";

/// The application state object.
pub struct AppState {
    pub client: Client,
    pub config: Arc<Config>,
    pub db: Addr<MongoExecutor>,
}

pub fn new_app(db: Addr<MongoExecutor>, client: Client, config: Arc<Config>) -> Addr<Server> {
    let app = server::new(move || {
        let state = AppState{
            client: client.clone(),
            config: config.clone(),
            db: db.clone(),
        };

        App::with_state(state)
            .scope("/api", |scope| {
                scope.resource("/register", |r| r.method(Method::POST).with(handlers::register))
                    .resource("/login", |r| r.method(Method::POST).with(handlers::login))
                    .resource("/search_giphy", |r| r.method(Method::POST).with(handlers::search_giphy))
                    .resource("/save_gif", |r| r.method(Method::POST).with(handlers::save_gif))
                    .resource("/favorites", |r| r.method(Method::POST).with(handlers::favorites))
            })

            // Build static file handler.
            .handler("/static/", StaticFiles::with_config(STATIC_DIR, StaticFileServerConfig)
                .expect("Failed to build static file handler. Probably means static dir is bad or there are FS issues.")
                .default_handler(|_req: &HttpRequest<AppState>| -> Result<NamedFile> {
                    let path = Path::new(STATIC_DIR).join("index.html");
                    Ok(NamedFile::open(path)?)
                }))

            .default_resource(|r| r.get().h(|_req: &HttpRequest<AppState>| -> Result<NamedFile> {
                let path = Path::new(STATIC_DIR).join("index.html");
                Ok(NamedFile::open(path)?)
            }))

            .finish()
    })
    .bind("127.0.0.1:8080") // TODO: 0.0.0.0
    .expect("Expected to bind 127.0.0.1:8080 successfully.")
    .start();

    info!("Server is listening on 127.0.0.1:8080.");
    app
}

/// The static file server configuration for this API.
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
