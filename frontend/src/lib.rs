#![allow(clippy::wildcard_imports)]

use seed::{*, prelude::*};

// Define and init application state
mod model;
// Render state into vDOM instance
mod view;
// Translate UI events into state changes
mod controller;

// Application state struct
use model::Model;
// Enum over handled events
use controller::Msg;

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", model::init, controller::update, view::view);
}
