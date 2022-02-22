use super::*;

mod root;
use root::*;
mod settings;
use settings::*;

// Model for underlying components
pub(crate) struct RoutesModel {
  root: RootModel,
  settings: SettingsModel,
}
impl RoutesModel {
  pub(crate) fn new() -> Self {
    Self {
      root: RootModel::new(),
      settings: SettingsModel::new(),
    }
  }
}

// Enum over handled callbacks for underlying components
pub(crate) enum RoutesMsg {
  Root(RootMsg),
  Settings(SettingsMsg),
}
// Callback handler for those callbacks
pub(crate) fn routes_update(
  msg: RoutesMsg,
  model: &mut RoutesModel,
  session: &shared_types::Session,
  orders: &mut impl Orders<Msg>,
) {
  match msg {
    RoutesMsg::Root(msg) => root_update(msg, &mut model.root, orders),
    RoutesMsg::Settings(msg) => settings_update(msg, &mut model.settings, session, orders),
  }
}

// Define the routes a string constants, to let the compiler prevent mis-spelling
const SETTINGS: &str = "settings";
//const ADMIN: &str = "admin";

pub(crate) fn routes_view(
  model: &RoutesModel,
  _session: &shared_types::Session,
  mut url: Url,
) -> Node<RoutesMsg> {
  // Match on first part of the path, handing down accordingly
  match url.next_hash_path_part() {
    None => root_view(&model.root).map_msg(|x| RoutesMsg::Root(x)),
    Some(SETTINGS) => settings_view(&model.settings).map_msg(|x| RoutesMsg::Settings(x)),
    //        Some(ADMIN) => admin::view(model),
    // If not an url we know, return a nice error page
    _ => bad_url(url),
  }
}
fn bad_url<X>(url: Url) -> Node<X> {
  div![C!["error"], format!("Given path not found: \"{}\".", url),]
}
