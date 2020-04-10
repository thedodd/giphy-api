// /// Handle search giphy requests.
// pub fn search_giphy(
//     state: State<AppState>, data: Json<SearchGiphyRequest>, jwt: AuthHeader,
// ) -> Box<dyn Future<Item=HttpResponse, Error=WebError>> {
//     // Validate the given JWT before processing request.
//     let user_id = match user_oid_from_auth(&state, jwt.0) {
//         Ok(user_id) => user_id,
//         Err(err) => return Box::new(ok(
//             HttpResponse::Ok().json(&Response::<SearchGiphyResponse>::Error(err))
//         )),
//     };

//     // Build future for fetching the user's saved GIFs.
//     let gifs_f = state.db.send(db::FindUserSavedGifs(user_id))
//         .then(flatten_mailbox_error)
//         .map(|gifs| gifs.into_iter().fold(HashMap::new(), |mut acc, gif| {
//             acc.insert(gif.giphy_id.clone(), gif);
//             acc
//         }));

//     // Fetch a payload of Gifs from Giphy according to the given search.
//     let query_f = gifs_f.and_then(move |gifs| {
//         state.client.get(GIPHY_SEARCH_URL)
//             .query(&[
//                 ("api_key", state.config.giphy_api_key.as_str()),
//                 ("q", data.query.as_str()),
//                 ("limit", "50"),
//             ]).send().and_then(|res| res.error_for_status())
//             .and_then(|mut result| {
//                 result.json::<GiphySearchResponse<Vec<GiphySearchGif>>>()
//             })
//             .then(move |res| match res {
//                 Ok(payload) => {
//                     let gifs = payload.data.into_iter().map(|gif| {
//                         let saved = gifs.get(&gif.id);
//                         GiphyGif{
//                             id: gif.id, title: gif.title, is_saved: saved.is_some(),
//                             url: gif.images.fixed_height_downsampled.url,
//                             category: saved.map(|gif| gif.category.clone()).unwrap_or_default(),
//                         }
//                     }).collect();
//                     Ok(SearchGiphyResponse{gifs})
//                 },
//                 Err(err) => {
//                     error!("Error from query to the Giphy API. {:?}", err);
//                     Err(Error::new_ise())
//                 }
//             })
//     })
//     .then(|res| match res {
//         Ok(data) => Ok(Response::Data(data)),
//         Err(err) => Ok(Response::Error(err)),
//     })
//     .map(|res| HttpResponse::Ok().json(res))
//     .map_err(|_: ()| -> WebError { unreachable!() });

//     Box::new(query_f)
// }

// /// Handle save GIF requests.
// pub fn save_gif(
//     state: State<AppState>, data: Json<SaveGifRequest>, jwt: AuthHeader,
// ) -> Box<dyn Future<Item=HttpResponse, Error=WebError>> {
//     // Validate the given JWT before processing request.
//     let user_id = match user_oid_from_auth(&state, jwt.0) {
//         Ok(user_id) => user_id,
//         Err(err) => return Box::new(ok(
//             HttpResponse::Ok().json(&Response::<SaveGifResponse>::Error(err))
//         )),
//     };

//     // Fetch the target GIF from the Giphy API.
//     let data = data.into_inner();
//     let giphy_f = state.client.get(&format!("{}/{}", GIPHY_ID_URL, &data.id))
//         .query(&[("api_key", state.config.giphy_api_key.as_str())]).send()
//         .and_then(|res| res.error_for_status())
//         .and_then(|mut result| {
//             result.json::<GiphySearchResponse<Option<GiphySearchGif>>>()
//         })
//         .then(move |res| match res {
//             Ok(payload) => {
//                 match payload.data {
//                     Some(gif) => Ok(GiphyGif{
//                         id: gif.id, title: gif.title, is_saved: false,
//                         url: gif.images.fixed_height_downsampled.url,
//                         category: None,
//                     }),
//                     None => Err(Error::new("Specified GIF does not seem to exist in Gipy.", 400, None)),
//                 }
//             },
//             Err(err) => {
//                 error!("Error from query to the Giphy API. {:?}", err);
//                 Err(Error::new_ise())
//             }
//         });

//     // Save the GIF to the DB for the user if we have a successful lookup.
//     let gif_f = giphy_f.and_then(move |gif: GiphyGif| {
//         state.db.send(db::SaveGif(user_id, gif)).then(flatten_mailbox_error)
//     })
//     .map(|saved_gif| SaveGifResponse{gif: GiphyGif::from(saved_gif)})
//     .then(|res: Result<SaveGifResponse, Error>| match res {
//         Ok(data) => Ok(Response::Data(data)),
//         Err(err) => Ok(Response::Error(err)),
//     })
//     .map(|res| HttpResponse::Ok().json(res))
//     .map_err(|_: ()| -> WebError { unreachable!() });

//     Box::new(gif_f)
// }

// /// Handle requests for fetching user's favorites.
// pub fn favorites(
//     state: State<AppState>, _data: Json<FetchFavoritesRequest>, jwt: AuthHeader,
// ) -> Box<dyn Future<Item=HttpResponse, Error=WebError>> {
//     // Validate the given JWT before processing request.
//     let user_id = match user_oid_from_auth(&state, jwt.0) {
//         Ok(user_id) => user_id,
//         Err(err) => return Box::new(ok(
//             HttpResponse::Ok().json(&Response::<SearchGiphyResponse>::Error(err))
//         )),
//     };

//     // Build future for fetching the user's saved GIFs.
//     let gifs_f = state.db.send(db::FindUserSavedGifs(user_id))
//         .then(flatten_mailbox_error)
//         .map(|gifs| gifs.into_iter().map(|gif| GiphyGif::from(gif)).collect())
//         .map(|gifs| FetchFavoritesResponse{gifs})
//         .then(|res| match res {
//             Ok(data) => Ok(Response::Data(data)),
//             Err(err) => Ok(Response::Error(err)),
//         })
//         .map(|res| HttpResponse::Ok().json(res))
//         .map_err(|_: ()| -> WebError { unreachable!() });

//     Box::new(gifs_f)
// }

// /// Handle requests to categorize a GIF.
// pub fn categorize(
//     state: State<AppState>, data: Json<CategorizeGifRequest>, jwt: AuthHeader,
// ) -> Box<dyn Future<Item=HttpResponse, Error=WebError>> {
//     // Validate the given JWT before processing request.
//     let user_id = match user_oid_from_auth(&state, jwt.0) {
//         Ok(user_id) => user_id,
//         Err(err) => return Box::new(ok(
//             HttpResponse::Ok().json(&Response::<CategorizeGifResponse>::Error(err))
//         )),
//     };

//     // Build future for fetching the user's saved GIFs.
//     let gif_f = state.db.send(db::CategorizeGif(user_id, data.into_inner()))
//         .then(flatten_mailbox_error)
//         .map(|gif| CategorizeGifResponse{gif: GiphyGif::from(gif)})
//         .then(|res| match res {
//             Ok(data) => Ok(Response::Data(data)),
//             Err(err) => Ok(Response::Error(err)),
//         })
//         .map(|res| HttpResponse::Ok().json(res))
//         .map_err(|_: ()| -> WebError { unreachable!() });

//     Box::new(gif_f)
// }


