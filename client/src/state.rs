use seed::prelude::*;

use crate::{
    net::{NetworkEvent, NetworkState},
    proto::api::ResponseFrame,
};

/// The root data model of this application.
#[derive(Clone, Default)]
pub struct Model {
    pub network: NetworkState,
    pub msg_rx_cnt: usize,
    pub msg_tx_cnt: usize,
    pub input_text: String,
    pub messages: Vec<String>,
}

/// The different types of events which may influence the application's state.
#[derive(Clone)]
pub enum ModelEvent {
    Network(NetworkEvent),
    ServerMsg(ResponseFrame),
    EditChange(String),
}

/// The application's state update handler.
pub fn update(msg: ModelEvent, mut model: &mut Model) -> Update<ModelEvent> {
    match msg {
        ModelEvent::Network(event) => {
            NetworkEvent::reducer(event, model)
        }
        ModelEvent::ServerMsg(msg) => {
            model.msg_rx_cnt += 1;
            model.messages.push(msg.id);
            Render.into()
        }
        ModelEvent::EditChange(input_text) => {
            model.input_text = input_text;
            Render.into()
        }
    }
}
