use common::{
    Error, Response,
    CategorizeGifRequest, CategorizeGifResponse,
    FetchFavoritesRequest, FetchFavoritesResponse,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
    SaveGifRequest, SaveGifResponse,
    SearchGiphyRequest, SearchGiphyResponse,
};
use lazy_static::lazy_static;
use seed::{Method, Request};
use serde::{Serialize, de::DeserializeOwned};

const AUTHZ: &str = "authorization";

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
pub async fn login(req: LoginRequest) -> Result<LoginResponse, Error> {
    api_post(LOGIN_URL.to_string(), req, None).await
}

/// Submit a register request.
pub async fn register(req: RegisterRequest) -> Result<RegisterResponse, Error> {
    api_post(REGISTER_URL.to_string(), req, None).await
}

/// Submit a search giphy request.
pub async fn search(req: SearchGiphyRequest, jwt: String) -> Result<SearchGiphyResponse, Error> {
    api_post(SEARCH_URL.to_string(), req, Some(jwt)).await
}

/// Submit a save GIF request.
pub async fn save_gif(req: SaveGifRequest, jwt: String) -> Result<SaveGifResponse, (String, Error)> {
    let gifid = req.id.clone();
    api_post(SAVE_GIF_URL.to_string(), req, Some(jwt))
        .await
        .map_err(move |err| (gifid, err))
}

/// Submit a request to fetch the caller's saved GIFs.
pub async fn favorites(req: FetchFavoritesRequest, jwt: String) -> Result<FetchFavoritesResponse, Error> {
    api_post(FAVORITES_URL.to_string(), req, Some(jwt)).await
}

/// Submit a request to categorize a GIF.
pub async fn categorize(req: CategorizeGifRequest, jwt: String) -> Result<CategorizeGifResponse, (String, Error)> {
    let gifid = req.id.clone();
    api_post(CATG_URL.to_string(), req, Some(jwt))
        .await
        .map_err(move |err| (gifid, err))
}

pub async fn api_post<T, D>(url: String, req: T, jwt: Option<String>) -> Result<D, Error>
    where
        T: Serialize,
        D: DeserializeOwned + std::fmt::Debug + 'static,
{
    let mut builder = Request::new(url).method(Method::Post);
    if let Some(jwt) = jwt {
        builder = builder.header(AUTHZ, &format!("bearer {}", jwt))
    }
    builder
        .send_json(&req)
        .fetch_json_data(std::convert::identity)
        .await
        // Flatten fetch util's inner result type.
        .unwrap_or_else(|res| res)
        // Handle errors related to API interfacing. Everything here is treated
        // as an opaque 500. The server will always return a 200 on the network
        // layer. The response body embeds detailed error info.
        .map_err(|err| {
            seed::log!("Error while attempting to communicate with API.\n{:?}", err);
            Error::new("Internal error.", 500, None)
        })
        // Unpack inner response type.
        .and_then(|res: Response<D>| match res {
            Response::Data(data) => Ok(data),
            Response::Error(err) => Err(err),
        })
}
