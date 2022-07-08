use seed::{prelude::*, *};
use crate::Msg;
use shared_types::{
  Session,
  AdminReturnableUser,
  UsersFilter,
  UsersOrder,
};

pub(crate) struct UsersListModel {
  list: Option<Vec<AdminReturnableUser>>,
  filter: UsersFilter,
}
impl UsersListModel {
  pub(crate) fn new() -> Self {
    Self {
      list: None,
      filter: UsersFilter::default(),
    }
  }
}

pub(crate) enum UsersListMsg {
  UpdateUsernameRegex(String),
  UpdateIdMte(String),
  UpdateIdLte(String),
}
pub(crate) fn users_list_update(
  msg: UsersListMsg,
  model: &mut UsersListModel,
  session: &Session,
  orders: &mut impl Orders<Msg>,
) {
  match msg {
  }
}

pub(crate) fn users_list_view(model: &UsersListModel) -> Node<UsersListMsg> {
  div![
    C!("admin-user-list"),
    form![
      table![ // ?
        tr![
          C!("admin-user-filter"),
          td![
            input![
              input_ev(Ev::Change, UsersListMsg::UpdateUsernameRegex),
              attrs!(At::Value => model.users_filter.username_regex),
            ],
          ],
          td![
            input![
              input_ev(Ev::Change, UsersListMsg::UpdateIdMte),
              attrs!(
                At::Value => model.users_filter.id_mte,
                At::Type => "number",
              ),
            ],
            input![
              input_ev(Ev::Change, UsersListMsg::UpdateIdLte),
              attrs!(
                At::Value => model.users_filter.id_lte,
                At::Type => "number",
              ),
            ],
          ],
          td![
            input![
              input_ev(Ev::Click, |_| UsersListMsg::ToggleAdminEq),
              attrs!(
                At::Value => model.users_filter.admin_eq,
                At::Type => "checkbox",
              ),
            ],
          ],
          td![
            input![
              input_ev(Ev::Click, |_| UsersListMsg::ToggleLockedEq),
              attrs!(
                At::Value => model.users_filter.locked_eq,
                At::Type => "checkbox",
              ),
            ],
          ],
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
