#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

mod routes;
use routes::*;

struct Model {
    url: Url,
    name: String,
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
        url: url,
        name: "world".to_string(),
    }
}

#[derive(Clone)]
enum Msg {
    UrlChanged(subs::UrlChanged),
    Update(String),
}
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => model.url = url,
        Msg::Update(new) => model.name = new,
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        "Top menu bar or something",
        br!(),
        route(model),
        br!(),
        "A footer maybe?"
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
