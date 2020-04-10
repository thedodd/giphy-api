use std::sync::Arc;

use actix_web::{App, HttpServer};
use actix_web::web::{self, Data, HttpRequest};
use actix_web::Result;
use bytes::Bytes;
use reqwest::Client;
use sqlx::PgPool;
use validator::Validate;

use common::{
    Error, Response,
    // GiphyGif,
    // CategorizeGifRequest, CategorizeGifResponse,
    // FetchFavoritesRequest, FetchFavoritesResponse,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
    // SaveGifRequest, SaveGifResponse,
    // SearchGiphyRequest, SearchGiphyResponse,
};
use crate::auth;
use crate::config::Config;
use crate::models;

// const STATIC_DIR: &str = "./static";

/// The application state object.
#[derive(Clone)]
pub struct State {
    pub db: PgPool,
    pub client: Client,
    pub config: Arc<Config>,
}

pub fn new(db: PgPool, client: Client, config: Arc<Config>) -> Result<actix_web::dev::Server> {
    let state = State{db, client, config: config.clone()};
    Ok(HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/api/register").route(web::post().to(register)))
            .service(web::resource("/api/login").route(web::post().to(login)))
    })
    .bind(format!("0.0.0.0:{}", config.port))?
    .run())
}

async fn register(_: HttpRequest, body: Bytes, state: Data<State>) -> Result<Response<RegisterResponse>> {
    let span = tracing::info_span!("/api/register");
    let _span = span.enter();

    // Extract JSON body, validate & prep data.
    let req: RegisterRequest = serde_json::from_slice(&body).map_err(Error::new_deser_err)?;
    req.validate().map_err(Error::from)?;
    let pwhash = auth::hash_pw(&req.password)?;
    let email = req.email.to_lowercase(); // Make things consistent.

    // Attempt to insert the new user record. If email is already in use, return an error indicating such.
    let mut tx = state.db.begin().await.map_err(Error::from)?;
    let user = models::User::insert(email, pwhash, &mut tx).await?;

    // Generate JWT for user, commit TX & respond.
    let token = auth::Claims::new_for_user(&state.config.encoding_key, user.id)?;
    tx.commit().await.map_err(Error::from)?;
    Ok(Response::Data(RegisterResponse(user.into_common(token))))
}

async fn login(_: HttpRequest, body: Bytes, state: Data<State>) -> Result<Response<LoginResponse>> {
    let span = tracing::info_span!("/api/login");
    let _span = span.enter();

    // Extract JSON body & prep the data for DB query.
    let req: LoginRequest = serde_json::from_slice(&body).map_err(Error::new_deser_err)?;
    let email = req.email.to_lowercase(); // Make things consistent.

    // Find user by given creds.
    let mut db = state.db.acquire().await.map_err(Error::from)?;
    let user = match models::User::find_by_email(email, &mut db).await? {
        Some(user) => user,
        // If no record was found, then the given creds were invalid.
        None => Err(Error::new_invalid_credentials())?,
    };
    auth::verify_user_pw(&user, &req.password)?; // Will 401 if invalid.

    // Create a JWT for the user and respond.
    let token = auth::Claims::new_for_user(&state.config.encoding_key, user.id)?;
    Ok(Response::Data(LoginResponse(user.into_common(token))))
}

//                     .resource("/search_giphy", |r| r.method(Method::POST).with(handlers::search_giphy))
//                     .resource("/save_gif", |r| r.method(Method::POST).with(handlers::save_gif))
//                     .resource("/favorites", |r| r.method(Method::POST).with(handlers::favorites))
//                     .resource("/categorize", |r| r.method(Method::POST).with(handlers::categorize))
//             })

//             // Build static file handler.
//             .handler("/static/", StaticFiles::with_config(STATIC_DIR, StaticFileServerConfig)
//                 .expect("Failed to build static file handler. Probably means static dir is bad or there are FS issues.")
//                 .default_handler(|_req: &HttpRequest<AppState>| -> Result<NamedFile> {
//                     let path = Path::new(STATIC_DIR).join("index.html");
//                     Ok(NamedFile::open(path)?)
//                 }))

//             .default_resource(|r| r.get().h(|_req: &HttpRequest<AppState>| -> Result<NamedFile> {
//                 let path = Path::new(STATIC_DIR).join("index.html");
//                 Ok(NamedFile::open(path)?)
//             }))
