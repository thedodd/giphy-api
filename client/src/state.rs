use seed::prelude::*;

use crate::{
    containers::{LoginContainer, LoginContainerEvent},
    net::{NetworkEvent, NetworkState},
};

/// The root data model of this application.
#[derive(Clone, Default)]
pub struct Model {
    /// The state of the network interface.
    pub network: NetworkState,
    /// The state of the login container.
    pub login: LoginContainer,
}

/// The different types of events which may influence the application's state.
#[derive(Clone)]
pub enum ModelEvent {
    Login(LoginContainerEvent),
    Network(NetworkEvent),
}

/// The application's state update handler.
pub fn update(msg: ModelEvent, model: &mut Model) -> Update<ModelEvent> {
    match msg {
        ModelEvent::Network(event) => NetworkEvent::reducer(event, model),
        ModelEvent::Login(event) => LoginContainerEvent::reducer(event, model),
    }
}
