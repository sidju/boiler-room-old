use super::*;
use serde::{Serialize, de::DeserializeOwned};
use hyper::header::HeaderValue;
use hyper::body::{aggregate, Buf};

pub fn json <T: Serialize + ?Sized> (data: &T) -> Result<Response, Error> {
  let mut re = Response::new(serde_json::to_string(data)?.into());
  re.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
  Ok(re)
}

pub fn get_header(
  req: &Request,
  header_name: &str,
) -> Result<Option<String>, Error> {
  Ok( match req.headers().get(header_name) {
    Some(val) => Some(val.to_str()?.to_string()),
    None => None,
  } )
}

pub async fn from_body <T: DeserializeOwned> (req: &mut Request) -> Result<T, Error> {
  // Verify content type
  let content_type = req.headers().get("Content-Type")
    .map(|x| x.to_str().unwrap_or(""))
  ;
  if Some("application/x-www-form-urlencoded") != content_type {
    return Err(Error::BadRequest(
      "Expected Content-Type to be 'application/x-www-form-urlencoded'".to_string()
    ));
  }
  // Try to parse
  let data: T = serde_urlencoded::from_reader(
    aggregate(req.body_mut())
      .await
      ?
      .reader()
  )?;
  Ok(data)
}

pub fn verify_method_path_end(
  path_vec: &Vec<String>,
  req: &Request,
  expected_method: &Method,
) -> Result<(), Error> {
  if !path_vec.is_empty() {
    Err(Error::PathNotFound(
      format!("{}", req.uri().path())
    ))
  }
  else if req.method() != expected_method {
    Err(Error::MethodNotFound(
      req.method().clone()
    ))
  }
  else {
    Ok(())
  }
}
