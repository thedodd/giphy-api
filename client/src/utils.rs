use std::borrow::Cow;

use common::Error;
use web_sys::Storage;

use crate::state::ModelEvent;

/// A method for handling common errors.
///
/// If an update is returned, it should be used; else, the caller will need to handle the error.
pub fn handle_common_errors(err: &Error) -> Option<ModelEvent> {
    match err.status {
        401 => Some(ModelEvent::Logout),
        _ => None,
    }
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

/// Set a session storage key.
pub fn del_session_item(key: &str) {
    let _ = get_session_storage()
        .map_err(|_| ())
        .and_then(|s| s.remove_item(key).map_err(|err| {
            seed::log!(format!("Failed to remove session storage key '{}': {:?}", key, err));
        }));
}

/// Get a handle to the window's session storage.
fn get_session_storage() -> Result<Storage, Cow<'static, str>> {
    let err_msg = "Could not access session storage.";
    match seed::window().session_storage() {
        Ok(opt) => opt.ok_or(Cow::Borrowed(err_msg)),
        Err(err) => match err.as_string() {
            Some(s) => Err(Cow::Owned(s)),
            None => Err(Cow::Borrowed(err_msg)),
        }
    }
}
