use std::collections::{BTreeMap, HashMap, HashSet};

use common::{
    Error, GiphyGif,
    CategorizeGifRequest, CategorizeGifResponse,
    FetchFavoritesRequest, FetchFavoritesResponse,
};
use seed::{*, prelude::*};

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
    pub category_updates: HashMap<String, String>,
    pub saving_category: HashSet<String>,
    pub fetch_error: Option<Error>,
    pub is_fetching_favorites: bool,
    pub filter: String,
}

impl Favorites {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.favorites.clear();
        self.category_updates.clear();
        self.saving_category.clear();
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
    Categorize(String),
    CategorizeSuccess(CategorizeGifResponse),
    CategorizeError(String, Error),
    UpdateFilter(String),
    UpdateCategory(String, String),
}

impl FavoritesEvent {
    /// The reducer for this state model.
    pub fn reducer(event: FavoritesEvent, mut model: &mut Model, orders: &mut impl Orders<ModelEvent>) {
        match event {
            FavoritesEvent::Fetch => match &model.user {
                Some(user) => {
                    model.favorites.is_fetching_favorites = true;
                    let jwt = user.jwt.clone();
                    orders.perform_cmd(async move {
                        api::favorites(FetchFavoritesRequest{}, jwt).await
                            .map(|data| ModelEvent::Favorites(FavoritesEvent::FetchSuccess(data)))
                            .map_err(|err| ModelEvent::Favorites(FavoritesEvent::FetchError(err)))
                    });
                }
                None => {
                    orders.send_msg(ModelEvent::Logout);
                }
            }
            FavoritesEvent::FetchSuccess(res) => {
                model.favorites.is_fetching_favorites = false;
                res.gifs.into_iter().for_each(|gif| { model.favorites.favorites.insert(gif.id.clone(), gif); });
            }
            FavoritesEvent::FetchError(err) => {
                model.favorites.is_fetching_favorites = false;
                model.favorites.fetch_error = Some(err.clone());
                if let Some(event) = handle_common_errors(&err) {
                    orders.send_msg(event);
                }
            }
            FavoritesEvent::Categorize(id) => match &model.user {
                Some(user) => match model.favorites.category_updates.get(&id) {
                    Some(category) => {
                        model.favorites.saving_category.insert(id.clone());
                        let payload = CategorizeGifRequest{id: id.clone(), category: category.to_string()};
                        let jwt = user.jwt.clone();
                        orders.perform_cmd(async move {
                            api::categorize(payload, jwt).await
                                .map(|data| ModelEvent::Favorites(FavoritesEvent::CategorizeSuccess(data)))
                                .map_err(|(id, e)| ModelEvent::Favorites(FavoritesEvent::CategorizeError(id, e)))
                        });
                    }
                    None => {
                        orders.skip();
                    }
                }
                None => {
                    orders.send_msg(ModelEvent::Logout);
                }
            },
            FavoritesEvent::CategorizeSuccess(res) => {
                let gif = res.gif;
                model.favorites.saving_category.remove(&gif.id);
                model.favorites.category_updates.remove(&gif.id);
                model.favorites.favorites.insert(gif.id.clone(), gif);
            }
            FavoritesEvent::CategorizeError(id, err) => {
                log!(format!("Error while saving category. {:?}", &err));
                model.favorites.saving_category.remove(&id);
            }
            FavoritesEvent::UpdateFilter(filter) => {
                model.favorites.filter = filter;
            }
            FavoritesEvent::UpdateCategory(id, val) => if val.len() > 0 {
                model.favorites.category_updates.insert(id, val);
            } else {
                model.favorites.category_updates.remove(&id);
            }
        }
    }
}

/// The favorites view.
pub fn favorites(model: &Model) -> Node<ModelEvent> {
    let spinner: Node<ModelEvent> = match model.favorites.is_fetching_favorites {
        true => span!(class!("icon ml-1"), i!(attrs!(At::Class => "fas fa-spinner fa-pulse"))),
        false => b!(""),
    };

    div!(attrs!{At::Class => "Favorites hero-body"; At::Id => "favorites"},
        div!(attrs!{At::Class => "container"},
            h1!(attrs!{At::Class => "title has-text-centered"}, "Favorites", spinner),
            div!(attrs!{At::Class => "field is-horizontal Favorites-field-container"},
                div!(attrs!{At::Class => "field-body"},
                    div!(attrs!{At::Class => "field is-expanded"},
                        div!(attrs!{At::Class => "field has-addons"},
                            p!(attrs!{At::Class => "control"},
                                // NB: due to a rendering bug in this framework, we need to be
                                // sure that this `button!` element type is different than the
                                // element type on the search page.
                                button!(attrs!{At::Class => "button is-static"},
                                    i!(attrs!{At::Class => "fas fa-filter"}),
                                ),
                            ),
                            p!(attrs!{At::Class => "control is-expanded"},
                                input!(
                                    attrs!(At::Value => model.favorites.filter; At::Class => "input"; At::Placeholder => "Filter by category"),
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
            div!(class!("columns is-1 is-mobile is-multiline is-centered Favorites-images"),
                model.favorites.favorites.values()
                    .filter(|gif| match model.favorites.filter.len() > 0 {
                        true => gif.category.as_ref()
                            .map(|catg| catg.contains(model.favorites.filter.as_str()))
                            .unwrap_or(false),
                        false => true,
                    })
                    .map(|gif|
                        gifcard(&gif, model.favorites.category_updates.get(&gif.id),
                            |_id| ModelEvent::Noop,
                            |_id| ModelEvent::Noop,
                            |id, catg| ModelEvent::Favorites(FavoritesEvent::UpdateCategory(id, catg)),
                            |id| ModelEvent::Favorites(FavoritesEvent::Categorize(id)),
                        )
                    ).collect::<Vec<_>>()
            )
        )
    )
}
