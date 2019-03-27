use common::GiphyGif;
use seed::prelude::*;

use crate::{
    state::ModelEvent,
};

/// A card displaying information on a Giphy GIF.
pub fn gifcard(gif: &GiphyGif, mut on_save: impl FnMut(String) -> ModelEvent + 'static) -> El<ModelEvent> {
    let icon = match gif.is_saved {
        true => span!(class!("icon is-size-6 has-text-warning"), i!(attrs!(At::Class => "fas fa-star"))),
        false => p!(class!("icon is-size-6 has-text-warning"), i!(attrs!(At::Class => "far fa-star"))),
    };
    let event = on_save(gif.id.clone());

    div!(class!("column is-full-mobile is-half-tablet is-one-quarter-desktop"),
        div!(class!("box p-three-quarter"),
            div!(class!("media justify-content-center"),
                div!(class!("media-center"),
                    div!(class!("content"),

                        figure!(class!("image"),
                            img!(attrs!("src" => &gif.url))
                        ),
                        p!(class!("mb-half is-size-7"), &gif.title),
                        button!(class!("button is-text is-small"),
                            simple_ev(Ev::Click, event),
                            icon
                        )

                    )
                )
            )
        )
    )
}
