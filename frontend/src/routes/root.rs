use super::*;

pub(crate) struct RootModel {
  name: String,
}
impl RootModel {
  pub(crate) fn new(orders: &mut impl Orders<Msg>) -> Self {
    Self {
      name: "".to_string(),
    }
  }
}

#[derive(Clone)]
pub(crate) enum RootMsg {
  UpdateName(String),
}
pub(crate) fn root_update(msg: RootMsg, model: &mut Model, orders: &mut impl Orders<Msg>) {
  match msg {
    RootMsg::UpdateName(x) => model.routes.root.name = x,
  }
}

pub(crate) fn root_view(model: &Model) -> Node<Msg> {
  let name = model.routes.root.name.clone();
  div![
    C!["hello"],
    "Hello ",
    &name,
    "!",
    br!(),
    input![
      input_ev(Ev::Change, |x| Msg::Routes(RoutesMsg::Root(
        RootMsg::UpdateName(x)
      )),),
      attrs!(At::Value => name)
    ],
  ]
}
