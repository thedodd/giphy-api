use common::User;
use seed::prelude::*;

use crate::{
    containers::{
        LoginContainer, LoginContainerEvent,
        SearchContainer, SearchContainerEvent,
    },
    router::Route,
    ui::{UIState, UIStateEvent}
};

/// The root data model of this application.
#[derive(Default)]
pub struct Model {
    /// A flag to indicate if the app is initializing.
    pub is_initializing: bool,

    /// The app's current route.
    pub route: Route,

    /// The state of various controlled UI components.
    pub ui: UIState,

    /// The state of the currently logged in user.
    pub user: Option<User>,

    /// The state of the login container.
    pub login: LoginContainer,

    /// The state of the search container.
    pub search: SearchContainer,
}

impl Model {
    /// Revert the model back to a pristine state.
    pub fn pristine(&mut self) {
        self.user = None;
        self.is_initializing = false;
        self.login.pristine();
        self.search.pristine();
        self.ui.pristine();
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
            route.push();
            model.route = route;
            Render.into()
        }
        ModelEvent::Login(event) => LoginContainerEvent::reducer(event, model),
        ModelEvent::Search(event) => SearchContainerEvent::reducer(event, model),
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
