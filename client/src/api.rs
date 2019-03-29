use common::{
    Error, Response,
    CategorizeGifRequest, CategorizeGifResponse,
    FetchFavoritesRequest, FetchFavoritesResponse,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
    SaveGifRequest, SaveGifResponse,
    SearchGiphyRequest, SearchGiphyResponse,
};
use futures::prelude::*;
use lazy_static::lazy_static;
use seed::{Method, Request};
use web_sys::console::log_1;

const AUTHZ: &str = "authorization";
const CTYPE: &str = "content-type";
const APP_JSON: &str = "application/json";
const FUT_ERROR: &str = "Error from request to API.";

lazy_static! {
    static ref BASE_URL: String = {
        let l = seed::window().location();
        let (proto, host) = (l.protocol().unwrap(), l.host().unwrap());
        format!("{}//{}/api", proto, host)
    };
    static ref REGISTER_URL: String = format!("{}/register", &*BASE_URL);
    static ref LOGIN_URL: String = format!("{}/login", &*BASE_URL);
    static ref SEARCH_URL: String = format!("{}/search_giphy", &*BASE_URL);
    static ref SAVE_GIF_URL: String = format!("{}/save_gif", &*BASE_URL);
    static ref FAVORITES_URL: String = format!("{}/favorites", &*BASE_URL);
    static ref CATG_URL: String = format!("{}/categorize", &*BASE_URL);
}

/// Submit a login request.
pub fn login(req: LoginRequest) -> impl Future<Item=LoginResponse, Error=Error> {
    Request::new(&*LOGIN_URL).method(Method::Post).header(CTYPE, APP_JSON)
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
}

/// Submit a register request.
pub fn register(req: RegisterRequest) -> impl Future<Item=RegisterResponse, Error=Error> {
    Request::new(&*REGISTER_URL).method(Method::Post).header(CTYPE, APP_JSON)
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
}

/// Submit a search giphy request.
pub fn search(req: SearchGiphyRequest, jwt: String) -> impl Future<Item=SearchGiphyResponse, Error=Error> {
    Request::new(&*SEARCH_URL).method(Method::Post)
        .header(CTYPE, APP_JSON).header(AUTHZ, &format!("bearer {}", jwt))
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
}

/// Submit a save GIF request.
pub fn save_gif(req: SaveGifRequest, jwt: String) -> impl Future<Item=SaveGifResponse, Error=(String, Error)> {
    Request::new(&*SAVE_GIF_URL).method(Method::Post)
        .header(CTYPE, APP_JSON).header(AUTHZ, &format!("bearer {}", jwt))
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
        .map_err(move |err| (req.id.clone(), err))
}

/// Submit a request to fetch the caller's saved GIFs.
pub fn favorites(req: FetchFavoritesRequest, jwt: String) -> impl Future<Item=FetchFavoritesResponse, Error=Error> {
    Request::new(&*FAVORITES_URL).method(Method::Post)
        .header(CTYPE, APP_JSON).header(AUTHZ, &format!("bearer {}", jwt))
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
}

/// Submit a request to categorize a GIF.
pub fn categorize(req: CategorizeGifRequest, jwt: String) -> impl Future<Item=CategorizeGifResponse, Error=(String, Error)> {
    Request::new(&*CATG_URL).method(Method::Post)
        .header(CTYPE, APP_JSON).header(AUTHZ, &format!("bearer {}", jwt))
        .body_json(&req).fetch_json().map_err(|err| {
            log_1(&err);
            Error::new(FUT_ERROR, 500, None)
        }).then(flatten_payload)
        .map_err(move |err| (req.id.clone(), err))
}

/// Flatten the result of an API response.
fn flatten_payload<D>(outer: Result<Response<D>, Error>) -> Result<D, Error> {
    match outer {
        Ok(inner) => match inner {
            Response::Data(data) => Ok(data),
            Response::Error(err) => Err(err),
        }
        Err(err) => Err(err),
    }
}
