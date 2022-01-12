use super::*;

mod routes;
use routes::*;

pub(crate) fn view(model: &Model) -> Node<Msg> {
    div![
        "Top menu bar or something",
        br!(),
        route(model),
        br!(),
        "A footer maybe?"
    ]
}

