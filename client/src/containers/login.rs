use common::{
    Error, User,
    LoginRequest, LoginResponse,
    RegisterRequest, RegisterResponse,
};
use seed::{*, prelude::*};
use validator::{validate_email};

use crate::api;
use crate::router::Route;
use crate::state::{Model, ModelEvent};
use crate::utils::set_session_item;

const NBSP: &str = " "; // Is not a space, it is an NBSP;
const EMAIL_ERR: &str = "Must provide a valid email address.";
const PASSWD_ERR: &str = "Password must be at least 6 characters in length.";

/// The state of the login container.
#[derive(Default)]
pub struct LoginContainer {
    pub email: String,
    pub email_error: Option<&'static str>,
    pub pw: String,
    pub pw_error: Option<&'static str>,
    pub network_error: Option<Error>,
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

    /// Clear any errors on this model.
    fn clear_errors(&mut self) {
        self.email_error = None;
        self.pw_error = None;
        self.network_error = None;
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
    pub fn reducer(event: LoginContainerEvent, mut model: &mut Model, orders: &mut impl Orders<ModelEvent>) {
        match event {
            LoginContainerEvent::UpdateEmailField(email) => {
                model.login.clear_errors();
                model.login.email = email;
            }
            LoginContainerEvent::UpdatePWField(pw) => {
                model.login.clear_errors();
                model.login.pw = pw;
            }
            LoginContainerEvent::Login => {
                model.login.clear_errors();
                model.login.has_network_request = true;
                let payload = LoginRequest{email: model.login.email.clone(), password: model.login.pw.clone()};
                orders.perform_cmd(async move {
                    api::login(payload).await
                        .map(|data| ModelEvent::Login(LoginContainerEvent::LoginSuccess(data)))
                        .map_err(|err| ModelEvent::Login(LoginContainerEvent::LoginError(err)))
                });
            }
            LoginContainerEvent::Register => {
                // Validate current input before submitting.
                if model.login.email_error.is_some() || model.login.pw_error.is_some() || model.login.has_network_request {
                    orders.skip();
                    return
                }
                if !validate_email(&model.login.email) && model.login.email.len() > 0 {
                    model.login.email_error = Some(EMAIL_ERR);
                    return
                }
                if model.login.pw.len() < 6 {
                    model.login.pw_error = Some(PASSWD_ERR);
                    return
                }

                // Evertying is ready to rock, so submit the registration request.
                model.login.clear_errors();
                model.login.has_network_request = true;
                let payload = RegisterRequest{email: model.login.email.clone(), password: model.login.pw.clone()};
                orders.perform_cmd(async move {
                    api::register(payload).await
                        .map(|data| ModelEvent::Login(LoginContainerEvent::RegisterSuccess(data)))
                        .map_err(|err| ModelEvent::Login(LoginContainerEvent::RegisterError(err)))
                });
            }
            LoginContainerEvent::LoginSuccess(res) => Self::handle_creds_response(res.0, model, orders),
            LoginContainerEvent::RegisterSuccess(res) => Self::handle_creds_response(res.0, model, orders),
            LoginContainerEvent::LoginError(err) => Self::handle_error(err, model),
            LoginContainerEvent::RegisterError(err) => Self::handle_error(err, model),
        }
    }

    fn handle_error(err: Error, mut model: &mut Model) {
        model.login.has_network_request = false;
        model.login.network_error = Some(err);
    }

    fn handle_creds_response(user: User, mut model: &mut Model, orders: &mut impl Orders<ModelEvent>) {
        let user_json = serde_json::to_string(&user).unwrap(); // This will never fail.
        let _ = set_session_item("user", &user_json).map_err(|err| {
            log!(format!("{} User info will not be persisted in session storage.", &err));
        });
        model.user = Some(user);
        orders.send_msg(ModelEvent::Route(Route::Search));
    }
}

/// The login view.
pub fn login(model: &Model) -> Node<ModelEvent> {
    let is_email_success = model.login.email.len() > 0 && model.login.email_error.is_none();
    let is_pw_success = model.login.pw.len() > 0;
    let button_attrs = match is_email_success && is_pw_success {
        true => attrs!{At::Class => "button is-dark is-outlined"},
        false => attrs!{At::Class => "button is-dark is-outlined"; At::Disabled => true},
    };
    let spinner = match model.login.has_network_request {
        true => span!(class!("icon"), i!(attrs!(At::Class => "fas fa-spinner fa-pulse"))),
        false => seed::empty(),
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
                        div!(attrs!{At::Class => "columns is-mobile is-vcentered"},
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
                            ),
                            div!(attrs!{At::Class => "column is-narrow"}, spinner)
                        )
                    )
                )
            ),
            p!(attrs!{At::Class => "control"},
                model.login.email_error
                    .or(model.login.pw_error)
                    .or(model.login.network_error.as_ref().map(|e| e.description.as_str()))
                    .unwrap_or(NBSP))
        )
    )
}
