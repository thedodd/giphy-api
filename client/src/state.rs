use common::User;
use seed::prelude::*;

use crate::{
    containers::{
        Favorites, FavoritesEvent,
        LoginContainer, LoginContainerEvent,
        SearchContainer, SearchContainerEvent,
    },
    router::Route,
    ui::{UIState, UIStateEvent},
    utils,
};

/// The root data model of this application.
#[derive(Default)]
pub struct Model {
    pub is_initializing: bool,
    pub route: Route,
    pub ui: UIState,
    pub user: Option<User>,
    pub login: LoginContainer,
    pub search: SearchContainer,
    pub favorites: Favorites,
}

impl Model {
    /// Revert the model back to a pristine state.
    pub fn pristine(&mut self) {
        self.is_initializing = false;
        self.ui.pristine();
        self.user = None;
        self.login.pristine();
        self.search.pristine();
        self.favorites.pristine();
    }
}

/// The different types of events which may influence the application's state.
#[derive(Clone)]
pub enum ModelEvent {
    Noop,
    Logout,
    Route(Route),
    Login(LoginContainerEvent),
    Search(SearchContainerEvent),
    Favorites(FavoritesEvent),
    Initialized(Option<User>),
    UI(UIStateEvent),
}

/// The application's state update handler.
pub fn update(msg: ModelEvent, model: &mut Model, orders: &mut impl Orders<ModelEvent>) {
    match msg {
        ModelEvent::Noop => {
            orders.skip();
        }
        ModelEvent::Logout => {
            model.pristine();
            utils::del_session_item("user");
            orders.send_msg(ModelEvent::Route(Route::Login));
        }
        ModelEvent::Route(route) => {
            if model.is_initializing {
                orders.skip();
                return;
            }
            model.ui.is_navbar_burger_active = false;
            route.push();
            model.route = route.clone();
            if let Some(event) = route.post_routing() {
                orders.send_msg(event);
            }
        }
        ModelEvent::Login(event) => LoginContainerEvent::reducer(event, model, orders),
        ModelEvent::Search(event) => SearchContainerEvent::reducer(event, model, orders),
        ModelEvent::Favorites(event) => FavoritesEvent::reducer(event, model, orders),
        ModelEvent::Initialized(user_opt) => {
            let has_session = user_opt.is_some();
            model.is_initializing = false;
            model.user = user_opt;

            // If user needs to login, push them to the login page.
            if has_session {
                seed::log!("User has a session, pushing to search page.");
                orders.send_msg(ModelEvent::Route(Route::Search));
            } else {
                seed::log!("User has no session, pushing to login page.");
                orders.send_msg(ModelEvent::Route(Route::Login));
            }
        }
        ModelEvent::UI(event) => UIStateEvent::reducer(event, model, orders),
    }
}
