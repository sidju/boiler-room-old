use super::*;

pub(crate) struct Model {
    pub(crate) url: Url,
    pub(crate) name: String,
}

pub(crate) fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    Model {
        url: url,
        name: "world".to_string(),
    }
}
