use hyper::{Body, Response};

pub trait Reply {
  fn into_response(self) -> Response<Body>;
}

impl Reply for Response<Body> {
  fn into_response(self) -> Response<Body> {
    self
  }
}

impl<T, E> Reply for Result<T, E>
where
  T: Reply,
  E: Reply,
{
  fn into_response(self) -> Response<Body> {
    match self {
      Ok(re) => re.into_response(),
      Err(e) => e.into_response(),
    }
  }
}
