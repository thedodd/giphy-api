use std::collections::BTreeMap;

use common::{
    Error, GiphyGif,
    FetchFavoritesRequest, FetchFavoritesResponse,
};
use futures::prelude::*;
use seed::prelude::*;

use crate::{
    api,
    components::gifcard,
    state::{Model, ModelEvent},
    utils::handle_common_errors,
};

/// The state of the favorites container.
#[derive(Default)]
pub struct Favorites {
    pub favorites: BTreeMap<String, GiphyGif>,
    pub fetch_error: Option<Error>,
    pub is_fetching_favorites: bool,
    pub filter: String,
}

impl Favorites {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.favorites.clear();
        self.fetch_error = None;
        self.is_fetching_favorites = false;
        self.filter = String::new();
    }
}

/// The set of events which may come from this container.
#[derive(Clone)]
pub enum FavoritesEvent {
    Fetch,
    FetchSuccess(FetchFavoritesResponse),
    FetchError(Error),
    Categorize(String, String),
    CategorizeSuccess(GiphyGif),
    CategorizeError(String, Error),
    UpdateFilter(String),
}

impl FavoritesEvent {
    /// The reducer for this state model.
    pub fn reducer(event: FavoritesEvent, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            FavoritesEvent::Fetch => match &model.user {
                Some(user) => {
                    model.favorites.is_fetching_favorites = true;
                    Update::with_future_msg(api::favorites(FetchFavoritesRequest, user.jwt.clone())
                        .map(|r| ModelEvent::Favorites(FavoritesEvent::FetchSuccess(r)))
                        .map_err(|e| ModelEvent::Favorites(FavoritesEvent::FetchError(e))))
                }
                None => Update::with_msg(ModelEvent::Logout),
            }
            FavoritesEvent::FetchSuccess(res) => {
                model.favorites.is_fetching_favorites = false;
                res.gifs.into_iter().for_each(|gif| { model.favorites.favorites.insert(gif.id.clone(), gif); });
                Render.into()
            }
            FavoritesEvent::FetchError(err) => {
                model.favorites.is_fetching_favorites = false;
                model.favorites.fetch_error = Some(err.clone());
                handle_common_errors(&err).unwrap_or(Render.into())
            }
            FavoritesEvent::Categorize(_id, _catg) => Skip.into(),
            FavoritesEvent::CategorizeSuccess(_gif) => Skip.into(),
            FavoritesEvent::CategorizeError(_id, _err) => Skip.into(),
            FavoritesEvent::UpdateFilter(_filter) => Skip.into(),
        }
    }
}

/// The favorites view.
pub fn favorites(model: &Model) -> El<ModelEvent> {
    let spinner: El<ModelEvent> = match model.favorites.is_fetching_favorites {
        true => span!(class!("icon ml-1"), i!(attrs!(At::Class => "fas fa-spinner fa-pulse"))),
        false => b!(""),
    };

    div!(attrs!{At::Class => "Favorites hero-body"; At::Id => "favorites"},
        div!(attrs!{"class" => "container"},
            h1!(attrs!{"class" => "title has-text-centered"}, "Favorites", spinner),
            div!(attrs!{"class" => "field is-horizontal Favorites-field-container"},
                div!(attrs!{"class" => "field-body"},
                    div!(attrs!{"class" => "field is-expanded"},
                        div!(attrs!{"class" => "field has-addons"},
                            p!(attrs!{"class" => "control"},
                                // NB: due to a rendering bug in this framework, we need to be
                                // sure that this `button!` element type is different than the
                                // element type on the search page.
                                button!(attrs!{"class" => "button is-static"},
                                    i!(attrs!{"class" => "fas fa-filter"}),
                                ),
                            ),
                            p!(attrs!{"class" => "control is-expanded"},
                                input!(
                                    attrs!{At::Value => model.favorites.filter; At::Class => "input"; At::PlaceHolder => "Filter by category"},
                                    input_ev(Ev::Input, |val| ModelEvent::Favorites(FavoritesEvent::UpdateFilter(val))),
                                ),
                            ),
                        ),
                        p!(class!("help is-size-6"), "Filter your saved GIFs by category. Leave blank to show all your saved GIFs."),
                        p!(class!("help is-size-6 has-text-weight-semibold has-text-danger"), model.favorites.fetch_error.as_ref().map(|e| e.description.as_str()).unwrap_or(" "))
                    )
                )
            ),

            // Search results will go here.
            div!(class!("columns is-1 is-mobile is-multiline Favorites-images"),
                model.favorites.favorites.values().map(|gif|
                    gifcard(&gif,
                        move |_id| ModelEvent::Noop,
                        move |_id| ModelEvent::Noop,
                        move |id, catg| ModelEvent::Favorites(FavoritesEvent::Categorize(id, catg)),
                    )
                ).collect::<Vec<_>>()
            )
        )
    )
}
