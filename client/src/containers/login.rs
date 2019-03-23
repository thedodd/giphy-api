use std::borrow::Cow;

use seed::prelude::*;
use validator::{validate_email};

use crate::{
    net::NetworkEvent,
    proto::api::{LoginResponse, RegisterResponse, RequestFrame},
    state::{Model, ModelEvent},
};

/// The state of the login container.
#[derive(Clone, Default)]
pub struct LoginContainer {
    pub email: String,
    pub email_error: Option<Cow<'static, str>>,
    pub pw: String,
    pub pw_error: Option<Cow<'static, str>>,
}

/// The set of events which may come from this container.
#[derive(Clone)]
pub enum LoginContainerEvent {
    UpdateEmailField(String),
    UpdatePWField(String),
    Login,
    LoginResponse(LoginResponse),
    Register,
    RegisterResponse(RegisterResponse),
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
                let req = RequestFrame::login(model.login.email.clone(), model.login.pw.clone());
                Update::with_msg(ModelEvent::Network(NetworkEvent::SendRequest(req)))
            }
            LoginContainerEvent::LoginResponse(res) => {
                log!(format!("Handling login response. {:?}", res));
                Skip.into()
            }
            LoginContainerEvent::Register => {
                let req = RequestFrame::register(model.login.email.clone(), model.login.pw.clone());
                Update::with_msg(ModelEvent::Network(NetworkEvent::SendRequest(req)))
            }
            LoginContainerEvent::RegisterResponse(res) => {
                log!(format!("Handling registration response. {:?}", res));
                Skip.into()
            }
        }
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

    section!(attrs!{"class" => "hero is-success is-fullheight"},
        div!(attrs!{"class" => "hero-body"},
            div!(attrs!{"class" => "container"},
                h1!(attrs!{"class" => "title has-text-centered"}, "GIPHY Client"),
                h5!(attrs!{"class" => "subtitle has-text-centered"}, "G-rated"),

                // Login / registration form.
                div!(attrs!{"class" => "Login"},
                    div!(attrs!{"class" => "columns"},
                        div!(attrs!{"class" => "column"},
                            div!(attrs!{"class" => "field"},
                                p!(attrs!{"class" => "control has-icons-left has-icons-right"},
                                    input!(
                                        attrs!{At::Value => model.login.email; "class" => "input"; "type" => "email"; "placeholder" => "Email";},
                                        input_ev(Ev::Input, |val| ModelEvent::Login(LoginContainerEvent::UpdateEmailField(val))),
                                    ),
                                    span!(attrs!{"class" => "icon is-small is-left"},
                                        i!(attrs!{"class" => "fas fa-envelope"}),
                                    ),
                                ),
                            ),
                            div!(attrs!{"class" => "field"},
                                p!(attrs!{"class" => "control has-icons-left"},
                                    input!(
                                        attrs!{At::Value => model.login.pw; At::Class => "input"; "type" => "password"; "placeholder" => "Password";},
                                        input_ev(Ev::Input, |val| ModelEvent::Login(LoginContainerEvent::UpdatePWField(val))),
                                    ),
                                    span!(attrs!{"class" => "icon is-small is-left"},
                                        i!(attrs!{"class" => "fas fa-lock"}),
                                    ),
                                ),
                            ),
                            div!(attrs!{"class" => "field"},
                                div!(attrs!{"class" => "columns is-mobile"},
                                    div!(attrs!{"class" => "column is-narrow"},
                                        button!(
                                            &button_attrs,
                                            simple_ev(Ev::Click, ModelEvent::Login(LoginContainerEvent::Login)),
                                            "Login"
                                        )
                                    ),
                                    div!(attrs!{"class" => "column is-narrow"},
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
                    p!(attrs!{"class" => "control"},
                        model.login.email_error.as_ref()
                            .or(model.login.pw_error.as_ref())
                            .unwrap_or(&Cow::Borrowed(" "))) // Is an &NBSP;
                )
            )
        )
    )
}
