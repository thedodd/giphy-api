use common::GiphyGif;
use seed::prelude::*;

use crate::{
    state::ModelEvent,
};

/// A card displaying information on a Giphy GIF.
pub fn gifcard(
    gif: &GiphyGif,
    catg_input: Option<&String>,
    mut on_save: impl FnMut(String) -> ModelEvent + 'static,
    mut on_remove: impl FnMut(String) -> ModelEvent + 'static,
    mut on_update_category: impl FnMut(String, String) -> ModelEvent + 'static,
    mut on_categorize: impl FnMut(String) -> ModelEvent + 'static,
) -> El<ModelEvent> {

    // Build closures.
    let (gid0, gid1, gid2, gid3) = (gif.id.clone(), gif.id.clone(), gif.id.clone(), gif.id.clone());
    let save_event = mouse_ev(Ev::Click, move |_| on_save(gid0.clone()));
    let _remove_event = mouse_ev(Ev::Click, move |_| on_remove(gid1.clone())); // TODO: wire up remove.
    let categorize_event = input_ev(Ev::Blur, move |_| on_categorize(gid2.clone()));
    let update_catg_event = input_ev(Ev::Input, move |catg| on_update_category(gid3.clone(), catg));

    // Build icon for favorite status.
    let is_saved = gif.is_saved;
    let icon = match is_saved {
        true => button!(class!("button is-rounded is-small has-text-warning"), save_event, b!(attrs!(At::Class => "fas fa-star"))),
        false => a!(class!("button is-rounded is-outlined is-small has-text-warning"), save_event, i!(attrs!(At::Class => "far fa-star"))),
    };

    // Setup category controls.
    let category_val = catg_input.map(|c| c.as_str())
        .or(gif.category.as_ref().map(|c| c.as_str()))
        .unwrap_or("");
    let mut input_attrs = attrs!(
        At::Class=>"input is-small is-rounded"; At::Value=>category_val;
        At::Type=>"text"; At::PlaceHolder=>"Categorize...";
    );
    if !is_saved {
        input_attrs.add(At::Disabled, "true");
    }

    div!(class!("GifCard-card column is-full-mobile is-half-tablet is-one-quarter-desktop"),
        div!(class!("box p-three-quarter"),
            div!(class!("media justify-content-center"),
                div!(class!("media-center"),
                    div!(class!("content"),

                        figure!(class!("image"),
                            img!(attrs!("src" => &gif.url))
                        ),
                        p!(class!("mb-half is-size-7"), &gif.title),
                        div!(class!("field is-grouped is-grouped-centered"),
                            p!(class!("control"),
                                icon
                            ),
                            p!(class!("control"),
                                input!(input_attrs, update_catg_event, categorize_event)
                            ),
                        )

                    )
                )
            )
        )
    )
}
