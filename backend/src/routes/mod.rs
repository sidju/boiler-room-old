use std::convert::Infallible;
use hyper::{StatusCode, Method};

use crate::{State, Error, Reply};
use crate::auth::Permissions;
use crate::sqlx_order;

mod utils;
pub use utils::*;

mod api;

type Response = hyper::Response<hyper::Body>;
type Request = hyper::Request<hyper::Body>;

pub async fn handle_request(
  state: &'static State,
  req: Request,
) -> Result<Response, Infallible> {
  // Call inner, unwrap its opaque type to a Response<Body>
  Ok(
    route(state, req)
      .await
      .into_response()
  )
}

// We have an inner handler to allow returning our own types,
// converting them into responses in the outer handler
async fn route(
  state: &'static State,
  req: Request,
) -> Result<Response, Error> {
  // Match on path to send into API or HTML module
  let mut path_vec: Vec<String> = req.uri().path()
    .split('/')
    .rev()
    .map(|s| s.to_owned())
    .collect()
  ;

  // If this path is functional the first
  // part will be None or Some(""), so error on else
  match path_vec.pop().as_deref() {
    None | Some("") => (),
    Some(unexpected) => {
      return Err(Error::path_data_before_root(unexpected.to_string()));
    },
  };

  match path_vec.pop().as_deref() {
    Some("api") => {
      api::route(
        state,
        req,
        path_vec,
      ).await
    },
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      Ok(Response::new("Hello from root!".into()))
    },
    Some(_) => {
      Err(Error::path_not_found(&req))
    },
  }
}
