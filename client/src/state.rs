use common::User;
use seed::prelude::*;

use crate::{
    containers::{
        Favorites, FavoritesEvent,
        LoginContainer, LoginContainerEvent,
        SearchContainer, SearchContainerEvent,
    },
    router::Route,
    ui::{UIState, UIStateEvent}
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
pub fn update(msg: ModelEvent, model: &mut Model) -> Update<ModelEvent> {
    match msg {
        ModelEvent::Noop => Skip.into(),
        ModelEvent::Logout => {
            model.pristine();
            Update::with_msg(ModelEvent::Route(Route::Login))
        }
        ModelEvent::Route(route) => {
            if model.is_initializing {
                return Skip.into();
            }
            model.ui.is_navbar_burger_active = false;
            route.push();
            model.route = route.clone();
            route.post_routing().unwrap_or(Render.into())
        }
        ModelEvent::Login(event) => LoginContainerEvent::reducer(event, model),
        ModelEvent::Search(event) => SearchContainerEvent::reducer(event, model),
        ModelEvent::Favorites(event) => FavoritesEvent::reducer(event, model),
        ModelEvent::Initialized(user_opt) => {
            let has_session = user_opt.is_some();
            model.is_initializing = false;
            model.user = user_opt;

            // If user needs to login, push them to the login page.
            if has_session {
                log!("User has a session, pushing to search page.");
                Update::with_msg(ModelEvent::Route(Route::Search))
            } else {
                log!("User has no session, pushing to login page.");
                Update::with_msg(ModelEvent::Route(Route::Login))
            }
        }
        ModelEvent::UI(event) => UIStateEvent::reducer(event, model),
    }
}
