use super::*;

#[derive(Clone)]
pub(crate) enum Msg {
    UrlChanged(subs::UrlChanged),
    Update(String),
}
pub(crate) fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => model.url = url,
        Msg::Update(new) => model.name = new,
    }
}

