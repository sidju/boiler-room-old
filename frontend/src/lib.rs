// We attempt to increase modularity by handing down all the
// base parts of seed to each route, giving full control over
// state, callbacks and rendering.
// This may seem like a boilerplate-y imitation of react, but
// since each route has access to global state it offers some
// additional flexibility in return for the verbosity.

#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

mod login;
use login::*;
mod routes;
use routes::*;

// Define and init application state
struct Model {
  // Global state variables
  pub url: Url,
  pub session: Option<shared_types::Session>,
  // Login variables, superseed routes
  pub login: LoginModel,
  // Route specific state variables
  pub routes: RoutesModel,
}
impl Model {
  fn new(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
      url: url,
      session: None,
      login: LoginModel::new(),
      routes: RoutesModel::new(orders),
    }
  }
}

// Translate callbacks into state changes
enum Msg {
  // Truly global events
  UrlChanged(subs::UrlChanged),
  Auth(Option<shared_types::Session>),
  // Login view events
  Login(LoginMsg),
  // To make the code more modular we
  // split events based on origin view
  Routes(RoutesMsg),
}
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
  match msg {
    Msg::UrlChanged(subs::UrlChanged(url)) => model.url = url,
    // Since session is an option this can log out
    Msg::Auth(new_session) => model.session = new_session,
    Msg::Login(msg) => login_update(msg, &mut model.login, orders),
    // For other routes, hand down events
    Msg::Routes(msg) => routes_update(msg, model, orders),
  }
}

// Render state into vDOM instance with callbacks
fn view(model: &Model) -> Node<Msg> {
  div![
    "Top menu bar----------",
    br!(),
    match &model.session {
      None => login_view(&model.login),
      Some(_session) => routes_view(model),
    },
    br!(),
    "Footer----------------"
  ]
}

#[wasm_bindgen(start)]
pub fn start() {
  // Mount the `app` to the element with the `id` "app".
  App::start("app", Model::new, update, view);
}
