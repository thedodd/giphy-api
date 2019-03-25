use seed::prelude::*;

use crate::{
    net::NetworkEvent,
    proto::api::{self, RequestFrame, SearchGiphyResponse},
    state::{
        Model, ModelEvent,
    },
};

/// The state of the search container.
#[derive(Clone, Default)]
pub struct SearchContainer {
    pub search: String,
    pub search_error: Option<String>,
    pub search_results: Vec<api::GiphyGif>,
    pub is_awaiting_response: bool,
}

impl SearchContainer {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.search = String::from("");
        self.search_error = None;
        self.search_results.clear();
        self.is_awaiting_response = false;
    }
}

/// The set of events which may come from this container.
#[derive(Clone)]
pub enum SearchContainerEvent {
    UpdateSearchField(String),
    SubmitSearch,
    SearchResponse(SearchGiphyResponse),
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
                    model.search.is_awaiting_response = true;
                    Update::with_msg(ModelEvent::Network(NetworkEvent::SendRequest(
                        RequestFrame::search_giphy(model.search.search.clone(), user.jwt.clone())
                    )))
                }
                None => Skip.into()
            }
            SearchContainerEvent::SearchResponse(mut res) => {
                model.search.is_awaiting_response = false;
                match res.error {
                    Some(err) => { model.search.search_error = Some(err.description); }
                    None => { model.search.search_results.append(&mut res.gifs); }
                }
                Render.into()
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
    if model.search.is_awaiting_response {
        search_input_attrs.add(At::Disabled, "true");
        submit_button_attrs.add(At::Disabled, "true");
    }
    if !model.search.is_awaiting_response && model.search.search.len() == 0 {
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
                        p!(class!("help is-size-6"), "Enter a search query, term, or phrase to get started.")
                    )
                )
            ),

            // Search results will go here.
            div!(class!("columns is-mobile is-multiline Search-images"),
                model.search.search_results.iter().map(|gif| {
                    div!(class!("column is-half-mobile is-one-quarter-desktop"),
                        div!(class!("box"),
                            div!(class!("media"),
                                div!(class!("media-center"),
                                    div!(class!("content"),

                                        figure!(class!("image"),
                                            img!(attrs!("src" => &gif.url))
                                        ),
                                        p!(&gif.title)

                                    )
                                )
                            )
                        )
                    )
                }).collect::<Vec<_>>()
            )
        )
    )
}
