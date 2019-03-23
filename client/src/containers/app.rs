use seed::prelude::*;

use crate::{
    containers::login::login,
    // net::NetworkEvent,
    // proto::api::{LoginRequest, RequestFrame},
    state::{Model, ModelEvent},
};

/// The root application view.
pub fn app(model: &Model) -> El<ModelEvent> {
    // div![
    //     h1!["HEB App"],
    //     if model.network.connected {
    //         div![
    //             input![
    //                 attrs! {
    //                     "type"=>"text";
    //                     "id"=>"text";
    //                     At::Value => model.input_text;
    //                 },
    //                 input_ev(Ev::Input, ModelEvent::EditChange)
    //             ],
    //             button![
    //                 attrs! {"type"=>"button";"id"=>"send"},
    //                 simple_ev(
    //                     "click",
    //                     build_login_request(model.input_text.clone(), model.input_text.clone())
    //                 ),
    //                 "Login"
    //             ]
    //         ]
    //     } else {
    //         div![p![em!["Connecting..."]]]
    //     },
    //     render_messages(&model.messages),
    //     footer![
    //         if model.network.connected {
    //             p!["Connected"]
    //         } else {
    //             p!["Disconnected"]
    //         },
    //         p![format!("{} messages received", model.msg_rx_cnt)],
    //         p![format!("{} messages sent", model.msg_tx_cnt)]
    //     ]
    // ]
    login(model)
}

// fn render_messages(msgs: &[String]) -> El<ModelEvent> {
//     let msgs: Vec<_> = msgs.iter().map(|m| p![m]).collect();
//     div![msgs]
// }

// fn build_login_request(email: String, password: String) -> ModelEvent {
//     let req = RequestFrame::login(LoginRequest{email, password});
//     ModelEvent::Network(NetworkEvent::SendRequest(req))
// }
