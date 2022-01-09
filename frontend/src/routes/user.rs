use super::*;

pub(crate) fn view(model: &Model) -> Node<Msg> {
    let name = model.name.clone();
    div![
        C!["hello"],
        "Hello ",
        &name,
        "!",
        br!(),
        input![input_ev(Ev::Change, Msg::Update), attrs!(At::Value => name)],
    ]
}
