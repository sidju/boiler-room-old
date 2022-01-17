use super::*;

mod root;
use root::*;

// Model for underlying components
pub(crate) struct RoutesModel {
  root: RootModel,
}
impl RoutesModel {
  pub(crate) fn new(orders: &mut impl Orders<Msg>) -> Self {
    Self {
      root: RootModel::new(orders),
    }
  }
}

// Enum over handled callbacks for underlying components
#[derive(Clone)]
pub(crate) enum RoutesMsg {
  Root(RootMsg),
}
// Callback handler for those callbacks
pub(crate) fn routes_update(msg: RoutesMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
  match msg {
    RoutesMsg::Root(msg) => root_update(msg, model, orders),
  }
}

// Define the routes a string constants, to let the compiler prevent mis-spelling
const LOGIN: &str = "login";
const USER: &str = "user";
const ADMIN: &str = "admin";

pub(crate) fn routes_view(model: &Model) -> Node<Msg> {
  // Get a copy of url, to have mutable access to its internal iterator
  let mut url = model.url.clone();
  // Match on first part of the path, handing down accordingly
  match url.next_path_part() {
    None => root_view(model),
    //        Some(LOGIN) => login::view(model),
    //        Some(USER) => user::view(model),
    //        Some(ADMIN) => admin::view(model),
    // If not an url we know, return a nice error page
    _ => bad_url(url, model),
  }
}
fn bad_url(url: Url, model: &Model) -> Node<Msg> {
  div![C!["not_found"], format!("Given URL not found:\"{}\".", url),]
}
