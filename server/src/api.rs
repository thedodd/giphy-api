use std::sync::Arc;

use actix_files as fs;
use actix_web::{App, HttpServer};
use actix_web::web::{self, Data, HttpRequest};
use actix_web::Result;
use bytes::Bytes;
use futures::prelude::*;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use validator::Validate;

use common::{
    Error, Response, GiphyGif,
    CategorizeGifRequest, CategorizeGifResponse,
    FetchFavoritesRequest, FetchFavoritesResponse,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
    SaveGifRequest, SaveGifResponse,
    SearchGiphyRequest, SearchGiphyResponse,
};
use crate::auth;
use crate::config::Config;
use crate::models;

const STATIC_DIR: &str = "./static";
const GIPHY_ID_URL: &str = "https://api.giphy.com/v1/gifs"; // Must append `/{id}`.
const GIPHY_SEARCH_URL: &str = "https://api.giphy.com/v1/gifs/search";

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

            // RPC endpoints.
            .service(web::resource("/api/register").route(web::post().to(register)))
            .service(web::resource("/api/login").route(web::post().to(login)))
            .service(web::resource("/api/search_giphy").route(web::post().to(search_giphy)))
            .service(web::resource("/api/save_gif").route(web::post().to(save_gif)))
            .service(web::resource("/api/favorites").route(web::post().to(favorites)))
            .service(web::resource("/api/categorize").route(web::post().to(categorize)))

            // Static content handlers.
            .service(fs::Files::new("/static", STATIC_DIR)
                .index_file(format!("{}/index.html", STATIC_DIR))
                .use_etag(true)
                .default_handler(web::route().to(index))
            )
            .default_service(web::route().to(index))

    })
    .bind(format!("0.0.0.0:{}", config.port))?
    .run())
}

async fn index(_: HttpRequest) -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open(format!("{}/index.html", STATIC_DIR))?)
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

async fn search_giphy(hreq: HttpRequest, body: Bytes, state: Data<State>) -> Result<Response<SearchGiphyResponse>> {
    let span = tracing::info_span!("/api/search_giphy");
    let _span = span.enter();

    // Extract claims from request & fetch the corresponding user.
    let claims = auth::Claims::from_request(&hreq, &state.config.decoding_key).await?;
    let mut db = state.db.acquire().await.map_err(Error::from)?;
    let user = match models::User::find_by_id(claims.sub, &mut db).await? {
        Some(user) => user,
        None => Err(Error::new_invalid_credentials())?,
    };
    let req: SearchGiphyRequest = serde_json::from_slice(&body).map_err(Error::new_deser_err)?;

    // Fetch a payload of Gifs from Giphy according to the given search.
    let res = state.client.get(GIPHY_SEARCH_URL)
        .query(&[
            ("api_key", state.config.giphy_api_key.as_str()),
            ("q", req.query.as_str()),
            ("limit", "50"),
        ])
        .send().await.map_err(|err| {
            tracing::error!("{}", err);
            Error::new_ise()
        })?
        .json::<GiphySearchResponse<Vec<GiphySearchGif>>>().await.map_err(|err| {
            tracing::error!("{}", err);
            Error::new_ise()
        })?;

    // Accumulate IDs for saved gifs search, unforuntaely the allocations are needed due to SQLX interface `:/`
    let search_gif_ids: Vec<String> = res.data.iter().map(|gif| gif.id.clone()).collect();

    // Query for all gifs already saved by the user matching the IDs of the gifs in the search payload.
    let saved_gifs = models::SavedGif::for_user_matching_ids(user.id, search_gif_ids.as_slice(), &mut db).await?;

    // Build response payload.
    let mut response_gifs = vec![];
    for gif in res.data {
        let saved = saved_gifs.get(&gif.id);
        response_gifs.push(GiphyGif{
            id: gif.id, title: gif.title, is_saved: saved.is_some(),
            url: gif.images.fixed_height_downsampled.url,
            category: saved.map(|gif| gif.category.clone()).unwrap_or_default(),
        });
    }

    Ok(Response::Data(SearchGiphyResponse{gifs: response_gifs}))
}

async fn save_gif(hreq: HttpRequest, _body: Bytes, state: Data<State>) -> Result<Response<SaveGifResponse>> {
    let span = tracing::info_span!("/api/save_gif");
    let _span = span.enter();
    let claims = auth::Claims::from_request(&hreq, &state.config.decoding_key).await?;
    // Save the gif by gif ID & user ID.
    unimplemented!()
}

async fn favorites(hreq: HttpRequest, _body: Bytes, state: Data<State>) -> Result<Response<FetchFavoritesResponse>> {
    let span = tracing::info_span!("/api/favorites");
    let _span = span.enter();
    let claims = auth::Claims::from_request(&hreq, &state.config.decoding_key).await?;
    // Just find all gifs saved/favorited by the user by ID.
    unimplemented!()
}

async fn categorize(hreq: HttpRequest, _body: Bytes, state: Data<State>) -> Result<Response<CategorizeGifResponse>> {
    let span = tracing::info_span!("/api/categorize");
    let _span = span.enter();
    let claims = auth::Claims::from_request(&hreq, &state.config.decoding_key).await?;
    // Set a new value for the gif previously saved by the user.
    unimplemented!()
}

#[derive(Deserialize, Serialize)]
struct GiphySearchResponse<D> {
    pub data: D,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGif {
    pub id: String,
    pub title: String,
    pub images: GiphySearchGifImages,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGifImages {
    pub fixed_height_downsampled: GiphySearchGifImagesModel,
}

#[derive(Deserialize, Serialize)]
struct GiphySearchGifImagesModel {
    #[serde(alias="mp4")]
    pub url: String,
}
