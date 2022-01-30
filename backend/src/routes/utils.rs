use super::*;
use serde::{de::DeserializeOwned, Serialize};

pub fn set_status(re: Result<Response, Error>, status: StatusCode) -> Result<Response, Error> {
  re.map(|mut r| {
    *r.status_mut() = status;
    r
  })
}

pub fn empty() -> Result<Response, Error> {
  let mut re = Response::new("".into());
  *re.status_mut() = StatusCode::NO_CONTENT;
  Ok(re)
}

pub fn not_modified() -> Result<Response, Error> {
  let mut re = Response::new("".into());
  *re.status_mut() = StatusCode::NOT_MODIFIED;
  Ok(re)
}
pub fn html(data: &'static str) -> Result<Response, Error> {
  let mut re = Response::new(data.into());
  re.headers_mut()
    .insert("Content-Type", HeaderValue::from_static("text/html; charset=utf-8"));
  Ok(re)
}
pub fn javascript(data: &'static str) -> Result<Response, Error> {
  let mut re = Response::new(data.into());
  re.headers_mut()
    .insert("Content-Type", HeaderValue::from_static("text/javascript; charset=utf-8"));
  Ok(re)
}
pub fn webassembly(data: &'static [u8]) -> Result<Response, Error> {
  let mut re = Response::new(data.into());
  re.headers_mut()
    .insert("Content-Type", HeaderValue::from_static("application/wasm"));
  Ok(re)
}

pub fn json<T: Serialize + ?Sized>(data: &T) -> Result<Response, Error> {
  let mut re = Response::new(serde_json::to_string(data)?.into());
  re.headers_mut()
    .insert("Content-Type", HeaderValue::from_static("application/json; charset=utf-8"));
  Ok(re)
}

pub fn get_header<'a>(req: &'a Request, header_name: &str) -> Result<Option<&'a str>, Error> {
  Ok(match req.headers().get(header_name) {
    Some(val) => Some(
      val
        .to_str()
        .map_err(|e| Error::unreadable_header(e, header_name))?,
    ),
    None => None,
  })
}

pub fn validate_get_content_len<'a>(req: &'a Request, max_len: usize) -> Result<usize, Error> {
  let header = get_header(&req, "Content-Length")?;
  if let Some(x) = header {
    let length = x.parse::<usize>().map_err(Error::content_length_not_int)?;
    if length <= max_len {
      Ok(length)
    } else {
      Err(Error::content_length_too_large(length, max_len))
    }
  } else {
    Err(Error::content_length_missing())
  }
}

// Serde performs better with continuous existing memory,
// so this is the most performant solution
pub async fn get_body(req: &mut Request, max_len: usize) -> Result<Vec<u8>, Error> {
  use hyper::body::HttpBody;

  // First we validate and set up
  let expected_len = validate_get_content_len(req, max_len)?;
  let mut bytes = Vec::with_capacity(expected_len);
  let body = req.body_mut();
  futures::pin_mut!(body);

  // Then we loop until we either overshoot Content-Len and error or
  // run out of data and return what we got
  while let Some(result) = body.data().await {
    let data = result?;
    // Check against overrunning
    if bytes.len() + data.len() > expected_len {
      // If we overrun try to estimate length of received request
      let estimate = bytes.len() + data.len() + body.size_hint().lower() as usize;
      return Err(Error::content_length_mismatch(estimate, expected_len));
    }

    bytes.extend_from_slice(&data);
  }

  // Finally check against undershooting
  if bytes.len() < expected_len {
    Err(Error::content_length_mismatch(bytes.len(), expected_len))
  } else {
    Ok(bytes)
  }
}

pub async fn parse_json<T: DeserializeOwned>(
  req: &mut Request,
  max_len: usize,
) -> Result<T, Error> {
  // Verify content type
  let content_type = get_header(req, "Content-Type")?.unwrap_or("");
  if "application/json; charset=utf-8" != content_type {
    return Err(Error::invalid_content_type(
      "application/json; charset=utf-8",
      content_type,
    ));
  }
  // Get body
  let bytes = get_body(req, max_len).await?;
  // Try to parse
  let data: T = serde_json::from_slice(&bytes)?;
  Ok(data)
}
pub fn parse_filter<T: DeserializeOwned>(req: &Request) -> Result<T, Error> {
  let query_str = req.uri().query().unwrap_or("");
  let filter: T = serde_urlencoded::from_str(query_str)?;
  Ok(filter)
}

pub fn verify_path_end(path_vec: &Vec<String>, req: &Request) -> Result<(), Error> {
  if !path_vec.is_empty() {
    Err(Error::path_not_found(req))
  } else {
    Ok(())
  }
}
pub fn verify_method(req: &Request, expected_method: &Method) -> Result<(), Error> {
  if req.method() != expected_method {
    Err(Error::method_not_found(req))
  } else {
    Ok(())
  }
}
pub fn verify_method_path_end(
  path_vec: &Vec<String>,
  req: &Request,
  expected_method: &Method,
) -> Result<(), Error> {
  verify_path_end(path_vec, req)?;
  verify_method(req, expected_method)?;
  Ok(())
}

// Unwrap a key expecting bearer auth type
pub fn unwrap_bearer(key: Option<&str>) -> Option<String> {
  // Unwrap Option
  match key {
    Some(ke) => {
      // Unwrap if 0..7 is valid selection
      let prefix = ke.get(..7).map(|k| k.to_ascii_lowercase());
      match prefix.as_ref().map(|s| &s[..]) {
        Some("bearer ") => ke.get(7..).map(|k| k.trim().to_string()),
        Some(_) => None,
        None => None,
      }
    }
    None => None,
  }
}
