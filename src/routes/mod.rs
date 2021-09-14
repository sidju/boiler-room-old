use std::convert::Infallible;
use hyper::{Body, Request, Response};
use crate::Reply;

pub async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  // Call inner, unwrap its opaque type to a Response<Body>
  Ok(
    inner_handle_request(req)
      .await
      .into_response()
  )
}

// We have an inner handler to allow returning our own types,
// converting them into responses in the outer handler
async fn inner_handle_request(_req: Request<Body>) -> impl Reply {
  Response::new("Hello there!".into())
}
