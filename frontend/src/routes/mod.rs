use super::*;

mod root;
use root::*;
mod settings;
use settings::*;
mod admin;
use admin::*;

// Model for underlying components
pub(crate) struct RoutesModel {
  root: RootModel,
  settings: SettingsModel,
  admin: AdminModel,
}
impl RoutesModel {
  pub(crate) fn new() -> Self {
    Self {
      root: RootModel::new(),
      settings: SettingsModel::new(),
      admin: AdminModel::new(),
    }
  }
}

// Enum over handled callbacks for underlying components
pub(crate) enum RoutesMsg {
  Root(RootMsg),
  Settings(SettingsMsg),
  Admin(AdminMsg),
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
    RoutesMsg::Admin(msg) => admin_update(msg, &mut model.admin, session, orders),
  }
}

pub(crate) fn routes_view(
  model: &RoutesModel,
  _session: &shared_types::Session,
  mut url: Url,
) -> Node<RoutesMsg> {
  // Match on first part of the path, handing down accordingly
  match url.next_hash_path_part() {
    None => root_view(&model.root).map_msg(|x| RoutesMsg::Root(x)),
    Some("settings") => settings_view(&model.settings).map_msg(|x| RoutesMsg::Settings(x)),
    Some("admin") => admin_view(&model.admin).map_msg(|x| RoutesMsg::Admin(x)),
    // If not an url we know, return a nice error page
    _ => bad_url(url),
  }
}
fn bad_url<X>(url: Url) -> Node<X> {
  div![C!["error"], format!("Given path not found: \"{}\".", url),]
}
