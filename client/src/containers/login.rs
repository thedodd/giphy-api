use std::borrow::Cow;

use common::{
    Error, User,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
};
use futures::prelude::*;
use seed::prelude::*;
use validator::{validate_email};

use crate::{
    api,
    router::Route,
    state::{Model, ModelEvent},
    utils::{set_session_item}
};

/// The state of the login container.
#[derive(Default)]
pub struct LoginContainer {
    pub email: String,
    pub email_error: Option<Cow<'static, str>>,
    pub pw: String,
    pub pw_error: Option<Cow<'static, str>>,
    pub network_error: Option<String>,
    pub has_network_request: bool,
}

impl LoginContainer {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.email = String::from("");
        self.email_error = None;
        self.pw = String::from("");
        self.pw_error = None;
        self.network_error = None;
        self.has_network_request = false;
    }
}

/// The set of events which may come from this container.
#[derive(Clone)]
pub enum LoginContainerEvent {
    UpdateEmailField(String),
    UpdatePWField(String),
    Login,
    LoginSuccess(LoginResponse),
    LoginError(Error),
    Register,
    RegisterSuccess(RegisterResponse),
    RegisterError(Error),
}

impl LoginContainerEvent {
    /// The reducer for this state model.
    pub fn reducer(event: LoginContainerEvent, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            LoginContainerEvent::UpdateEmailField(email) => {
                match validate_email(&email) {
                    false if email.len() > 0 => { model.login.email_error = Some(Cow::Borrowed("Must provide a valid email address.")); }
                    _ => { model.login.email_error = None; }
                }
                model.login.email = email;
                Render.into()
            }
            LoginContainerEvent::UpdatePWField(pw) => {
                match pw.len() >= 6 {
                    false => { model.login.pw_error = Some(Cow::Borrowed("Password must be at least 6 characters in length.")); }
                    true => { model.login.pw_error = None; }
                }
                model.login.pw = pw;
                Render.into()
            }
            LoginContainerEvent::Login => {
                model.login.has_network_request = true;
                let payload = LoginRequest{email: model.login.email.clone(), password: model.login.pw.clone()};
                Update::with_future_msg(api::login(payload)
                    .map(|r| ModelEvent::Login(LoginContainerEvent::LoginSuccess(r)))
                    .map_err(|e| ModelEvent::Login(LoginContainerEvent::LoginError(e))))
            }
            LoginContainerEvent::Register => {
                model.login.has_network_request = true;
                let payload = RegisterRequest{email: model.login.email.clone(), password: model.login.pw.clone()};
                Update::with_future_msg(api::register(payload)
                    .map(|r| ModelEvent::Login(LoginContainerEvent::RegisterSuccess(r)))
                    .map_err(|e| ModelEvent::Login(LoginContainerEvent::RegisterError(e))))
            }
            LoginContainerEvent::LoginSuccess(res) => Self::handle_creds_response(res.0, model),
            LoginContainerEvent::RegisterSuccess(res) => Self::handle_creds_response(res.0, model),
            LoginContainerEvent::LoginError(err) => Self::handle_error(err, model),
            LoginContainerEvent::RegisterError(err) => Self::handle_error(err, model),
        }
    }

    fn handle_error(err: Error, mut model: &mut Model) -> Update<ModelEvent> {
        model.login.has_network_request = false;
        model.login.network_error = Some(err.description);
        Render.into()
    }

    fn handle_creds_response(user: User, mut model: &mut Model) -> Update<ModelEvent> {
        let user_json = serde_json::to_string(&user).unwrap(); // This will never fail.
        let _ = set_session_item("user", &user_json).map_err(|err| {
            log!(format!("{} User info will not be persisted in session storage.", &err));
        });
        model.user = Some(user);
        Update::with_msg(ModelEvent::Route(Route::Search))
    }
}

/// The login view.
pub fn login(model: &Model) -> El<ModelEvent> {
    let is_email_success = model.login.email.len() > 0 && model.login.email_error.is_none();
    let is_pw_success = model.login.pw.len() >= 6;
    let button_attrs = match is_email_success && is_pw_success {
        true => attrs!{At::Class => "button is-dark is-outlined"},
        false => attrs!{At::Class => "button is-dark is-outlined"; At::Disabled => true},
    };

    div!(attrs!{At::Class => "hero-body Login"},
        div!(attrs!{At::Class => "container"},
            h1!(attrs!{At::Class => "title has-text-centered"}, "GIPHY Client"),
            h5!(attrs!{At::Class => "subtitle has-text-centered"}, "G-rated"),

            // Login / registration form.
            div!(attrs!{At::Class => "columns"},
                div!(attrs!{At::Class => "column"},
                    div!(attrs!{At::Class => "field"},
                        p!(attrs!{At::Class => "control has-icons-left has-icons-right"},
                            input!(
                                attrs!{At::Value => model.login.email; At::Class => "input"; "type" => "email"; "placeholder" => "Email";},
                                input_ev(Ev::Input, |val| ModelEvent::Login(LoginContainerEvent::UpdateEmailField(val))),
                            ),
                            span!(attrs!{At::Class => "icon is-small is-left"},
                                i!(attrs!{At::Class => "fas fa-envelope"}),
                            ),
                        ),
                    ),
                    div!(attrs!{At::Class => "field"},
                        p!(attrs!{At::Class => "control has-icons-left"},
                            input!(
                                attrs!{At::Value => model.login.pw; At::Class => "input"; "type" => "password"; "placeholder" => "Password";},
                                input_ev(Ev::Input, |val| ModelEvent::Login(LoginContainerEvent::UpdatePWField(val))),
                            ),
                            span!(attrs!{At::Class => "icon is-small is-left"},
                                i!(attrs!{At::Class => "fas fa-lock"}),
                            ),
                        ),
                    ),
                    div!(attrs!{At::Class => "field"},
                        div!(attrs!{At::Class => "columns is-mobile"},
                            div!(attrs!{At::Class => "column is-narrow"},
                                button!(
                                    &button_attrs,
                                    simple_ev(Ev::Click, ModelEvent::Login(LoginContainerEvent::Login)),
                                    "Login"
                                )
                            ),
                            div!(attrs!{At::Class => "column is-narrow"},
                                button!(
                                    &button_attrs,
                                    simple_ev(Ev::Click, ModelEvent::Login(LoginContainerEvent::Register)),
                                    "Register"
                                )
                            )
                        )
                    )
                )
            ),
            p!(attrs!{At::Class => "control"},
                model.login.email_error.as_ref()
                    .or(model.login.pw_error.as_ref())
                    .unwrap_or(&Cow::Borrowed(" "))) // Is a &NBSP;
        )
    )
}
