use seed::{*, prelude::*};

use crate::{
    components::navbar,
    containers::{favorites, login, search},
    router::Route,
    state::{Model, ModelEvent},
};

/// The root application view.
pub fn app(model: &Model) -> Vec<Node<ModelEvent>> {
    let route = &model.route;
    let nav = if route != &Route::Init && route != &Route::Login {
        navbar(model)
    } else {
        div!()
    };

    vec![
        section!(attrs!{At::Class => "hero is-success is-fullheight"},
            nav,

            match &model.route {
                Route::Init => div!(),
                Route::Login => login(model),
                Route::Search => search(model),
                Route::Favorites => favorites(model),
            }
        )
    ]
}
