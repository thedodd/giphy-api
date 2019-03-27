use std::collections::{BTreeMap, HashSet};

use common::{
    Error, GiphyGif,
    SaveGifRequest, SaveGifResponse,
    SearchGiphyRequest, SearchGiphyResponse,
};
use futures::prelude::*;
use seed::prelude::*;

use crate::{
    api,
    components::gifcard,
    state::{Model, ModelEvent},
    utils::handle_common_errors,
};

/// The state of the search container.
#[derive(Default)]
pub struct SearchContainer {
    pub search: String,
    pub search_error: Option<String>,
    pub search_results: BTreeMap<String, GiphyGif>,
    pub has_search_request: bool,
    pub gifs_being_saved: HashSet<String>,
}

impl SearchContainer {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.search = String::from("");
        self.search_error = None;
        self.search_results.clear();
        self.has_search_request = false;
        self.gifs_being_saved.clear();
    }
}

/// The set of events which may come from this container.
#[derive(Clone)]
pub enum SearchContainerEvent {
    UpdateSearchField(String),
    SubmitSearch,
    SearchSuccess(SearchGiphyResponse),
    SearchError(Error),
    SaveGif(String),
    SaveGifSuccess(SaveGifResponse),
    SaveGifError((String, Error)),
}

impl SearchContainerEvent {
    /// The reducer for this state model.
    pub fn reducer(event: SearchContainerEvent, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            SearchContainerEvent::UpdateSearchField(val) => {
                model.search.search = val;
                Render.into()
            }
            SearchContainerEvent::SubmitSearch => match &model.user {
                Some(user) => {
                    model.search.search_error = None;
                    model.search.search_results.clear();
                    model.search.has_search_request = true;
                    let payload = SearchGiphyRequest{query: model.search.search.clone()};
                    Update::with_future_msg(api::search(payload, user.jwt.clone())
                        .map(|r| ModelEvent::Search(SearchContainerEvent::SearchSuccess(r)))
                        .map_err(|e| ModelEvent::Search(SearchContainerEvent::SearchError(e))))
                }
                None => Update::with_msg(ModelEvent::Logout),
            }
            SearchContainerEvent::SearchSuccess(res) => {
                model.search.has_search_request = false;
                res.gifs.into_iter().for_each(|gif| { model.search.search_results.insert(gif.id.clone(), gif); });
                Render.into()
            }
            SearchContainerEvent::SearchError(err) => {
                model.search.has_search_request = false;
                handle_common_errors(&err).unwrap_or_else(|| {
                    model.search.search_error = Some(err.description);
                    Render.into()
                })
            }
            SearchContainerEvent::SaveGif(gifid) => match &model.user {
                Some(user) => {
                    model.search.gifs_being_saved.insert(gifid.clone());
                    let req = SaveGifRequest{id: gifid};
                    Update::with_future_msg(api::save_gif(req, user.jwt.clone())
                        .map(|r| ModelEvent::Search(SearchContainerEvent::SaveGifSuccess(r)))
                        .map_err(|e| ModelEvent::Search(SearchContainerEvent::SaveGifError(e))))
                }
                None => Update::with_msg(ModelEvent::Logout),
            }
            SearchContainerEvent::SaveGifSuccess(res) => {
                model.search.gifs_being_saved.remove(&res.gif.id);
                model.search.search_results.insert(res.gif.id.clone(), res.gif);
                Render.into()
            }
            SearchContainerEvent::SaveGifError((id, err)) => {
                model.search.gifs_being_saved.remove(&id);
                handle_common_errors(&err).unwrap_or(Skip.into())
            }
        }
    }
}

/// The search view.
pub fn search(model: &Model) -> El<ModelEvent> {
    let mut search_input_attrs = attrs!{
        At::Value => model.search.search; At::Class => "input"; At::PlaceHolder => "Search for GIFs";
    };
    let mut submit_button_attrs = attrs!{At::Class => "button"};
    if model.search.has_search_request {
        search_input_attrs.add(At::Disabled, "true");
        submit_button_attrs.add(At::Disabled, "true");
    }
    if !model.search.has_search_request && model.search.search.len() == 0 {
        submit_button_attrs.add(At::Disabled, "true");
    }

    div!(attrs!{"class" => "Search hero-body"},
        div!(attrs!{"class" => "container"},
            h1!(attrs!{"class" => "title has-text-centered"}, "Search"),
            div!(attrs!{"class" => "field is-horizontal Search-field-container"},
                div!(attrs!{"class" => "field-body"},
                    div!(attrs!{"class" => "field is-expanded"},
                        div!(attrs!{"class" => "field has-addons"},
                            p!(attrs!{"class" => "control"},
                                a!(attrs!{"class" => "button is-static"},
                                    i!(attrs!{"class" => "fas fa-search"}),
                                ),
                            ),
                            p!(attrs!{"class" => "control is-expanded"},
                                input!(search_input_attrs,
                                    input_ev(Ev::Input, |val| ModelEvent::Search(SearchContainerEvent::UpdateSearchField(val))),
                                ),
                            ),
                            p!(attrs!{"class" => "control"},
                                a!(submit_button_attrs,
                                    simple_ev(Ev::Click, ModelEvent::Search(SearchContainerEvent::SubmitSearch)),
                                    "Submit"
                                )
                            )
                        ),
                        p!(class!("help is-size-6"), "Enter a search query, term, or phrase to get started."),
                        p!(class!("help is-size-6 has-text-weight-semibold has-text-danger"), model.search.search_error.as_ref().map(|v| v.as_str()).unwrap_or(" "))
                    )
                )
            ),

            // Search results will go here.
            div!(class!("columns is-1 is-mobile is-multiline Search-images"),
                model.search.search_results.values().map(|gif|
                    gifcard(&gif, |id| ModelEvent::Search(SearchContainerEvent::SaveGif(id)))
                ).collect::<Vec<_>>()
            )
        )
    )
}
