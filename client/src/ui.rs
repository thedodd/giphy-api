use seed::prelude::*;

use crate::{
    state::{Model, ModelEvent},
};

/// An enumeration of the possible UI update events.
#[derive(Clone)]
pub enum UIStateEvent {
    ToggleNavbar,
}

impl UIStateEvent {
    /// The reducer for this state model.
    pub fn reducer(event: Self, mut model: &mut Model) -> Update<ModelEvent> {
        match event {
            UIStateEvent::ToggleNavbar => {
                model.ui.is_navbar_burger_active = !model.ui.is_navbar_burger_active;
                Render.into()
            }
        }
    }
}

/// A data model to represent the state of various UI components.
#[derive(Clone)]
pub struct UIState {
    pub is_navbar_burger_active: bool,
}

impl Default for UIState {
    fn default() -> Self {
        Self{
            is_navbar_burger_active: false,
        }
    }
}

impl UIState {
    /// Revert this model back to a pristine state.
    pub fn pristine(&mut self) {
        self.is_navbar_burger_active = false;
    }
}
