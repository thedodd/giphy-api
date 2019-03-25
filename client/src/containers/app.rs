use seed::prelude::*;

use crate::{
    components::navbar,
    containers::{login, search},
    router::Route,
    state::{Model, ModelEvent},
};

/// The root application view.
pub fn app(model: &Model) -> El<ModelEvent> {
    let route = &model.route;

    section!(attrs!{"class" => "hero is-success is-fullheight"},
        if route != &Route::Init && route != &Route::Login {
            navbar(model)
        } else {
            div!()
        },

        match &model.route {
            Route::Init => div!(),
            Route::Login => login(model),
            Route::Search => search(model),
            Route::Favorites => div!(),
        }
    )
}
