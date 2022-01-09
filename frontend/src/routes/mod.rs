use super::*;

mod admin;
mod login;
mod root;
mod user;

// Define the routes a string constants, to let the compiler prevent mis-spelling
const LOGIN: &str = "login";
const USER: &str = "user";
const ADMIN: &str = "admin";

pub(crate) fn route(model: &Model) -> Node<Msg> {
    // Get a copy of url, to have mutable access to its internal iterator
    let mut url = model.url.clone();
    // Match on first part of the path, handing down accordingly
    match url.next_path_part() {
        None => root::view(model),
        Some(LOGIN) => login::view(model),
        Some(USER) => user::view(model),
        Some(ADMIN) => admin::view(model),
        // If not an url we know, return a nice error page
        _ => bad_url(model),
    }
}

fn bad_url(model: &Model) -> Node<Msg> {
    let url = model.url.clone();
    div![C!["not_found"], format!("Given URL not found:\"{}\".", url),]
}
