use seed::prelude::*;

use crate::{
    proto::api::{LoginRequest, RequestFrame},
    state::{Model, ModelEvent},
};

/// The root application view.
pub fn view(model: &Model) -> El<ModelEvent> {
    div![
        h1!["HEB App"],
        if model.connected {
            div![
                input![
                    attrs! {
                        "type"=>"text";
                        "id"=>"text";
                        At::Value => model.input_text;
                    },
                    input_ev(Ev::Input, ModelEvent::EditChange)
                ],
                button![
                    attrs! {"type"=>"button";"id"=>"send"},
                    simple_ev(
                        "click",
                        ModelEvent::Send(RequestFrame::login(LoginRequest{
                            username: model.input_text.clone(),
                            password: model.input_text.clone(),
                        }))
                    ),
                    "Login"
                ]
            ]
        } else {
            div![p![em!["Connecting..."]]]
        },
        render_messages(&model.messages),
        footer![
            if model.connected {
                p!["Connected"]
            } else {
                p!["Disconnected"]
            },
            p![format!("{} messages received", model.msg_rx_cnt)],
            p![format!("{} messages sent", model.msg_tx_cnt)]
        ]
    ]
}

fn render_messages(msgs: &[String]) -> El<ModelEvent> {
    let msgs: Vec<_> = msgs.iter().map(|m| p![m]).collect();
    div![msgs]
}
