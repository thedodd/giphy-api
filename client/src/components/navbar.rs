use seed::prelude::*;

use crate::{
    router::Route,
    state::{Model, ModelEvent},
    ui::UIStateEvent,
};

/// The navbar component.
pub fn navbar(model: &Model) -> El<ModelEvent> {
    let mut navburger_attrs = attrs!("role" => "button"; "aria-label" => "menu"; "aria-expanded" => "false");
    let mut navmenu_attrs = attrs!();
    if model.ui.is_navbar_burger_active {
        navburger_attrs.add(At::Class, "navbar-burger is-active");
        navmenu_attrs.add(At::Class, "navbar-menu is-active")
    } else {
        navburger_attrs.add(At::Class, "navbar-burger");
        navmenu_attrs.add(At::Class, "navbar-menu")
    }

    // FUTURE: nice-to-have: wire up overlay for click-away.

    nav!(attrs!(At::Class => "Navbar navbar is-black is-fixed-top"),
        div!(attrs!(At::Class => "navbar-brand"),
            span!(attrs!(At::Class => "navbar-item"), b!("GIPHY Client")),

            a!(navburger_attrs, simple_ev(Ev::Click, ModelEvent::UI(UIStateEvent::ToggleNavbar)),
                span!(attrs!("aria-hidden" => "true")),
                span!(attrs!("aria-hidden" => "true")),
                span!(attrs!("aria-hidden" => "true")),
            )
        ),

        div!(navmenu_attrs,
            div!(attrs!(At::Class => "navbar-start"),
                div!(class!("navbar-item"), build_search_link(&model.route)),
                div!(class!("navbar-item"), build_favorites_link(&model.route)),
            ),
            div!(attrs!(At::Class => "navbar-end"),
                div!(attrs!(At::Class => "navbar-item"),
                    button!(class!("button is-dark"),
                        simple_ev(Ev::Click, ModelEvent::Logout),
                        "Logout"
                    )
                )
            )
        )
    )
}

fn build_search_link(route: &Route) -> El<ModelEvent> {
    let mut attrs = attrs!(At::Class => "button is-dark"; At::Href => "/ui/search");
    if let Route::Search = route {
        attrs.add(At::Disabled, "true");
    }
    a!(attrs, "Search")
}

fn build_favorites_link(route: &Route) -> El<ModelEvent> {
    let mut attrs = attrs!(At::Class => "button is-dark"; At::Href => "/ui/favorites");
    if let Route::Favorites = route {
        attrs.add(At::Disabled, "true");
    }
    a!(attrs, "Favorites")
}
