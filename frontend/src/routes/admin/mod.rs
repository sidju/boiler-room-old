use super::*;
use shared_types::{
  AdminReturnableUser,
  UsersFilter,
  UsersOrder,
};

pub(crate) struct AdminModel {
  users: Option<Vec<AdminReturnableUser>>,
  users_filter: UsersFilter,
}
impl AdminModel {
  pub(crate) fn new() -> Self {
    Self {
      users: None,
      users_filter: UsersFilter::default(),
    }
  }
}

pub(crate) enum AdminMsg {
}
pub(crate) fn admin_update(
  msg: AdminMsg,
  model: &mut AdminModel,
  session: &shared_types::Session,
  orders: &mut impl Orders<Msg>,
) {
  match msg {
  }
}

pub(crate) fn admin_view(model: &AdminModel) -> Node<AdminMsg> {
  div![
    C!("admin-user-list"),
    form![
      C!("admin-user-filter"),
      table![ // ?
        input![
          input_ev(Ev::Change, UsersListMsg::UpdateUsernameRegex),
          attrs!(At::Value => model.users_filter.username_regex),
        ],
        input![
          input_ev(Ev::Change,)
        ],
      ],
    ],
    match &model.users {
      None => div![C!("notice"), "Enter a query to load in users",],
      Some(list) => if list.is_empty() {
        div![C!("notice"), "No users matching filters",]
      } else {
        div!["Some users"]
//        for user in list {
//        }
      }
    }
  ]
}
