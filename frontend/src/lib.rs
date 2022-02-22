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
  fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Self::new(url)
  }
  fn new(url: Url) -> Model {
    let session = match LocalStorage::get("session") {
      Ok(s) => {
        let s: shared_types::Session = s; // Needed to declare expected type...
                                          // If it has expired, ignore it
        if s.until < chrono::Utc::now().naive_utc() {
          None
        } else {
          Some(s)
        }
      }
      Err(e) => {
        log!("Could not load session from storage", e);
        None
      }
    };
    Model {
      url: url,
      session: session,
      login: LoginModel::new(),
      routes: RoutesModel::new(),
    }
  }
}

// Translate callbacks into state changes
enum Msg {
  // Truly global events
  UrlChanged(subs::UrlChanged),
  SetAuth(shared_types::Session),
  ClearAuth(&'static str), // Logout message
  // Login view events
  Login(LoginMsg),
  Logout,
  // To make the code more modular we
  // split events based on origin view
  Routes(RoutesMsg),
}
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
  // Before each update, check if our session has expired and clear it
  // Doing this before requests saves the backend some unauth. requests
  match &model.session {
    Some(s) => {
      if s.until < chrono::Utc::now().naive_utc() {
        model.session = None;
      }
    }
    None => (),
  }

  match msg {
    Msg::UrlChanged(subs::UrlChanged(url)) => model.url = url,
    // S/R for session, with needed side effects
    Msg::SetAuth(new_session) => {
      match LocalStorage::insert("session", &new_session) {
        Ok(()) => (),
        Err(e) => log!("Could not save session to storage", e),
      }
      model.session = Some(new_session);
    }
    Msg::ClearAuth(message) => {
      match LocalStorage::remove("session") {
        Ok(()) => (),
        Err(e) => log!("Could not delete session from storage", e),
      }
      // Clear out all stored state except login form
      // (in case duplicate ClearAuth comes with latency)
      let login = model.login.clone();
      *model = Model::new(model.url.clone()); // Also clears session
      model.login = login;
      model.login.logout_message = message;
    }
    // Event forwarder for login events, and logout handler (here since related and small)
    Msg::Login(msg) => login_update(msg, &mut model.login, orders),
    Msg::Logout => match model.session.as_ref().map(|s| s.key.clone()) {
      Some(session_key) => {
        // Send the request to the backend to delete the session
        let req = Request::new("/api/logout")
          .method(Method::Post)
          .header(Header::bearer(session_key))
          .json(&());
        orders.perform_cmd(async {
          let res: Result<(), FetchError> = async {
            let resp = req?.fetch().await?;
            match resp.status().code {
              204 | 200 => (),
              _ => {
                let err: shared_types::ClientError = resp.json().await?;
                log!("API error in logout request", err);
              }
            };
            Ok(())
          }
          .await;
          match res {
            Ok(()) => (),
            Err(e) => log!("Error occurred in logout request", e),
          };
        });
        // Remove the local session no matter what backend returns,
        // since in worst case backend deletes it on expiration
        orders.send_msg(Msg::ClearAuth("Successfully logged out."));
        orders.skip(); // Since sent message will trigger render
      }
      None => (),
    },
    // For other routes, hand down events
    // Only handle if session is some, since these shouldn't be accessible if not signed in
    Msg::Routes(msg) => match &model.session {
      Some(session) => routes_update(msg, &mut model.routes, session, orders),
      None => (),
    },
  }
}

// Render state into vDOM instance with callbacks
fn view(model: &Model) -> Node<Msg> {
  match &model.session {
    None => login_view(&model.login).map_msg(Msg::Login),
    Some(session) => {
      div![
        div![
          C!("navbar"),
          span![C!("navbar-left"), a!["PageRoot", attrs![At::Href => "#"],],],
          span![
            C!("navbar-right"),
            if session.is_admin {
              a!["Admin", attrs![At::Href => "#admin"],]
            } else {
              Node::Empty
            },
            a!["Settings", attrs![At::Href => "#settings"],],
            a![
              "Logout",
              attrs![At::Href => "#"],
              ev(Ev::Click, |_| Msg::Logout),
            ],
          ],
        ],
        div![
          C!("route-contents"),
          routes_view(&model.routes, session, model.url.clone()).map_msg(Msg::Routes),
        ],
        div![C!("footer"), "footer contents",],
      ]
    }
  }
}

#[wasm_bindgen(start)]
pub fn start() {
  // Mount the `app` to the element with the `id` "app".
  App::start("app", Model::init, update, view);
}
