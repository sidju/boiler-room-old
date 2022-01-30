use super::*;

pub(crate) struct RootModel {
  name: String,
}
impl RootModel {
  pub(crate) fn new() -> Self {
    Self {
      name: "".to_string(),
    }
  }
}

#[derive(Clone)]
pub(crate) enum RootMsg {
  UpdateName(String),
}
pub(crate) fn root_update(msg: RootMsg, model: &mut RootModel, _orders: &mut impl Orders<Msg>) {
  match msg {
    RootMsg::UpdateName(x) => model.name = x,
  }
}

pub(crate) fn root_view(model: &RootModel) -> Node<RootMsg> {
  let name = model.name.clone();
  div![
    C!["hello"],
    "Hello ",
    &name,
    "!",
    br!(),
    input![
      input_ev(Ev::Change, |x| RootMsg::UpdateName(x) ),
      attrs!(At::Value => name)
    ],
  ]
}
