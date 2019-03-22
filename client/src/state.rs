use seed::prelude::*;

use crate::{
    json,
};

/// The root data model of this application.
#[derive(Clone, Default)]
pub struct Model {
    pub connected: bool,
    pub msg_rx_cnt: usize,
    pub msg_tx_cnt: usize,
    pub input_text: String,
    pub messages: Vec<String>,
}

/// The different types of events which may influence the application's state.
#[derive(Clone)]
pub enum ModelEvent {
    Connected,
    ServerMsg(json::ServerMsg),
    Send(json::ClientMsg),
    Sent,
    EditChange(String),
}

/// The application's state update handler.
pub fn update(msg: ModelEvent, mut model: &mut Model) -> Update<ModelEvent> {
    match msg {
        ModelEvent::Connected => {
            model.connected = true;
            Render.into()
        }
        ModelEvent::ServerMsg(msg) => {
            model.connected = true;
            model.msg_rx_cnt += 1;
            model.messages.push(msg.text);
            Render.into()
        }
        ModelEvent::EditChange(input_text) => {
            model.input_text = input_text;
            Render.into()
        }
        ModelEvent::Send(_) => Skip.into(),
        ModelEvent::Sent => {
            model.input_text = "".into();
            model.msg_tx_cnt += 1;
            Render.into()
        }
    }
}
