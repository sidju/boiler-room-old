use super::*;
use serde::{Serialize, de::DeserializeOwned};
use hyper::header::HeaderValue;
use hyper::body::{aggregate, Buf};

pub fn json <T: Serialize + ?Sized> (data: &T) -> Result<Response, Error> {
  let mut re = Response::new(serde_json::to_string(data)?.into());
  re.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
  Ok(re)
}

pub fn get_header<'a>(
  req: &'a Request,
  header_name: &str,
) -> Result<Option<&'a str>, Error> {
  Ok( match req.headers().get(header_name) {
    Some(val) => Some(val.to_str()?),
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

// Unwrap a key expecting bearer auth type
pub fn unwrap_bearer(
  key: Option<&str>,
) -> Option<String> {
  // Unwrap Option
  match key {
    Some(ke) => {
      // Unwrap if 0..7 is valid selection
      let prefix = ke.get(..7).map(|k| { k.to_ascii_lowercase() });
      match prefix.as_ref().map(|s| &s[..]) {
        Some("bearer ") => {
          ke.get(7..).map(|k| k.trim().to_string())
        },
        Some(_) => None,
        None => None,
      }
    },
    None => None,
  }
}
