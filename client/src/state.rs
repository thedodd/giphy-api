use std::borrow::Cow;

use seed::prelude::*;
use serde_derive::{Deserialize, Serialize};
use web_sys::{Window, Storage};

use crate::{
    containers::{
        LoginContainer, LoginContainerEvent,
        SearchContainer, SearchContainerEvent,
    },
    net::{NetworkEvent, NetworkState},
    router::Route,
};

/// The root data model of this application.
#[derive(Clone, Default)]
pub struct Model {
    /// The app's current route.
    pub route: Route,

    /// The state of the currently logged in user.
    pub user: Option<User>,

    /// A flag to indicate if the app is initializing.
    pub is_initializing: bool,

    /// The state of the network interface.
    pub network: NetworkState,

    /// The state of the login container.
    pub login: LoginContainer,

    /// The state of the search container.
    pub search: SearchContainer,

    /// The state of various controlled UI components.
    pub ui: UIState,
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
    Network(NetworkEvent),
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
        ModelEvent::Network(event) => NetworkEvent::reducer(event, model),
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

/// A model representing the currently logged in user.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub jwt: String,
}

/// Attempt to access a key from session storage.
pub fn get_session_item(key: &str) -> Result<String, Cow<'static, str>> {
    get_session_storage()
        .and_then(|s| s.get_item(key).map_err(|err| match err.as_string() {
            Some(s) => Cow::Owned(s),
            None => Cow::Borrowed("Failed to set session storage key."),
        }))
        .and_then(|opt| match opt {
            Some(s) => Ok(s),
            None => Err(Cow::Borrowed("Key not found in storage.")),
        })
}

/// Set a session storage key.
pub fn set_session_item(key: &str, val: &str) -> Result<(), Cow<'static, str>> {
    get_session_storage()
        .and_then(|s| s.set_item(key, val).map_err(|err| match err.as_string() {
            Some(s) => Cow::Owned(s),
            None => Cow::Borrowed("Failed to set session storage key."),
        }))
}

/// Get a handle to the window's session storage.
fn get_session_storage() -> Result<Storage, Cow<'static, str>> {
    web_sys::window().ok_or(Cow::Borrowed("Could not access window object."))
        .and_then(|w: Window| {
            let err_msg = "Could not access session storage.";
            match w.session_storage() {
                Ok(opt) => opt.ok_or(Cow::Borrowed(err_msg)),
                Err(err) => match err.as_string() {
                    Some(s) => Err(Cow::Owned(s)),
                    None => Err(Cow::Borrowed(err_msg)),
                }
            }
        })
}

/// An enumeration of the possible UI update events.
#[derive(Clone)]
pub enum UIStateEvent {
    ToggleNavbar,
}

impl UIStateEvent {
    /// The reducer for this state model.
    pub fn reducer(event: Self, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            UIStateEvent::ToggleNavbar => {
                model.ui.is_navbar_burger_active = !model.ui.is_navbar_burger_active;
                Render.into()
            }
        }
    }
}

/// A data model to represent the state of various UI components.
#[derive(Clone)]
pub struct UIState {
    pub is_navbar_burger_active: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self{
            is_navbar_burger_active: false,
        }
    }
}

impl UIState {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.is_navbar_burger_active = false;
    }
}
