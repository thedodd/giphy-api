use seed::Url;

use crate::{
    ModelEvent,
};

/// The application's routes.
#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Init,
    Login,
    Search,
    Favorites,
}

impl Default for Route {
    fn default() -> Self {
        Route::Init
    }
}

impl Route {
    /// Push the URL to the desired location.
    ///
    /// **NOTE WELL:** this should really only be called from the ModelEvent::Route handler.
    pub fn push(&self) {
        match self {
            Route::Init => seed::push_path(vec!["ui"]),
            Route::Login => seed::push_path(vec!["ui", "login"]),
            Route::Search => seed::push_path(vec!["ui", "search"]),
            Route::Favorites => seed::push_path(vec!["ui", "favorites"]),
        }
    }
}

/// This application's router function.
pub fn router(url: &Url) -> ModelEvent {
    // FUTURE: setup pre init `requested url` pattern, for being
    // able to route to requested page when login is needed.

    match url.path.get(0).unwrap().as_str() { // There is always a 0th element.
        "ui" => match url.path.get(1) {
            None => {
                log!("Routing to init page.");
                ModelEvent::Route(Route::Init)
            }
            Some(path) if path == "login" => {
                log!("Routing to login page.");
                ModelEvent::Route(Route::Login)
            }
            Some(path) if path == "search" => {
                log!("Routing to search page.");
                ModelEvent::Route(Route::Search)
            }
            Some(path) if path == "favorites" => {
                log!("Routing to favorites page.");
                ModelEvent::Route(Route::Favorites)
            }
            Some(path) => {
                log!("Unrecognized route '{}', sending to search page.", path);
                ModelEvent::Route(Route::Search)
            }
        }
        _ => {
            ModelEvent::Route(Route::Init)
        }
    }
}

// TODO: probably kill this.
// /// Get the current URl.
// ///
// /// **NB:** this technically has the possibility of panicing if we are not running in
// /// a browser, or if there is some critical error within web-sys land. Shouldn't happen though.
// pub fn get_url() -> Url {
//     let w = web_sys::window().expect("Failed to get a handle to the browser's window object.");
//     let d = w.document().expect("Failed to access the brower's current document.");
//     Url::from(d.url().expect("Failed to access the browser's current URL."))
// }
