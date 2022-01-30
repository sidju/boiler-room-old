// Error type for the whole project

// Needed types
use crate::Reply;
use argon2::password_hash;
use hyper::header::HeaderValue;
use hyper::{Body, Request, Response, StatusCode};
// Public errors to wrap
use hyper::header::ToStrError as UnreadableHeaderError;
use serde_json::Error as JsonError;
use serde_urlencoded::de::Error as UrlEncodingError;
use std::num::ParseIntError;
// Private errors to wrap
use hyper::Error as ConnectionError;
use password_hash::Error as HashingError;
use sqlx::Error as DbError;
use tokio::sync::AcquireError;
use tokio::task::JoinError;
// Client facing error type
use shared_types::ClientError;

// Then an object for private errors
// This only returns "internal server error" to user
#[derive(Debug)]
pub enum InternalError {
  SessionKeyCollision,
  Join(JoinError),
  Semaphore(AcquireError),
  Hash(HashingError),
  Db(DbError),
  Connection(ConnectionError),
}
impl Reply for InternalError {
  fn into_response(self) -> Response<Body> {
    eprintln!("{:?}", &self);
    let mut re = Response::new("Internal server error".into());
    *re.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    re.headers_mut()
      .insert("Content-Type", HeaderValue::from_static("application/json; charset=utf-8"));
    re
  }
}

impl Reply for ClientError {
  fn into_response(self) -> Response<Body> {
    let mut re = Response::new(serde_json::to_string(&self).unwrap().into());
    *re.status_mut() = match self {
      Self::PathNotFound(_) => StatusCode::NOT_FOUND,
      Self::MethodNotFound(_) => StatusCode::METHOD_NOT_ALLOWED,
      Self::Unauthorized => StatusCode::UNAUTHORIZED,
      Self::Forbidden => StatusCode::FORBIDDEN,

      Self::PathDataBeforeRoot(_) => StatusCode::BAD_REQUEST,
      Self::UnreadableHeader(_) => StatusCode::BAD_REQUEST,
      Self::InvalidContentLength(_) => StatusCode::BAD_REQUEST,
      Self::InvalidContentType(_) => StatusCode::BAD_REQUEST,
      Self::InvalidJson(_) => StatusCode::BAD_REQUEST,
      Self::InvalidUrlEncoding(_) => StatusCode::BAD_REQUEST,
      Self::InvalidIndexPath(_) => StatusCode::BAD_REQUEST,

      Self::BadPassword => StatusCode::BAD_REQUEST,
      Self::UsernameTaken => StatusCode::BAD_REQUEST,
      Self::BadLogin => StatusCode::UNAUTHORIZED,
      Self::AccountLocked => StatusCode::UNAUTHORIZED,
    };
    re.headers_mut()
      .insert("Content-Type", HeaderValue::from_static("application/json; charset=utf-8"));
    re
  }
}

// The Enum over either internal or client errors
// On this we implement From and relevant utilities
#[derive(Debug)]
pub enum Error {
  InternalError(InternalError),
  ClientError(ClientError),
}
// Utility constructors
impl Error {
  pub fn session_key_collision() -> Self {
    Self::InternalError(InternalError::SessionKeyCollision)
  }
  pub fn path_data_before_root(data: String) -> Self {
    Self::ClientError(ClientError::PathDataBeforeRoot(data))
  }
  pub fn path_not_found(req: &Request<Body>) -> Self {
    Self::ClientError(ClientError::PathNotFound(req.uri().path().to_owned()))
  }
  pub fn method_not_found(req: &Request<Body>) -> Self {
    Self::ClientError(ClientError::MethodNotFound(req.method().to_string()))
  }
  pub fn unauthorized() -> Self {
    Self::ClientError(ClientError::Unauthorized)
  }
  pub fn forbidden() -> Self {
    Self::ClientError(ClientError::Forbidden)
  }

  // Most of the parsing errors are created by From
  // but not these
  pub fn unreadable_header(e: UnreadableHeaderError, header: &str) -> Self {
    Self::ClientError(ClientError::UnreadableHeader(format!(
      "Error reading header {}: {}",
      header, e
    )))
  }
  pub fn content_length_missing() -> Self {
    Self::ClientError(ClientError::InvalidContentLength(
      "No content length given".to_string(),
    ))
  }
  pub fn content_length_not_int(err: ParseIntError) -> Self {
    Self::ClientError(ClientError::InvalidContentLength(format!(
      "Invalid unsigned int: {}",
      err
    )))
  }
  pub fn content_length_too_large(parsed: usize, max: usize) -> Self {
    Self::ClientError(ClientError::InvalidContentLength(format!(
      "Too large. Maximum allowed is {}, received {}",
      max, parsed
    )))
  }
  pub fn content_length_mismatch(given: usize, promised: usize) -> Self {
    let at_least = if given > promised { " at least" } else { "" };
    Self::ClientError(ClientError::InvalidContentLength(format!(
      "Mismatch. Header is {}, received{} {}",
      promised, at_least, given
    )))
  }
  pub fn invalid_content_type(parsed: &str, expected: &str) -> Self {
    Self::ClientError(ClientError::InvalidContentType(format!(
      "Expected {}, received {}",
      parsed, expected
    )))
  }

  // Finally the input processing errors
  pub fn bad_password() -> Self {
    Self::ClientError(ClientError::BadPassword)
  }
  pub fn username_taken() -> Self {
    Self::ClientError(ClientError::UsernameTaken)
  }
  pub fn bad_login() -> Self {
    Self::ClientError(ClientError::BadLogin)
  }
  pub fn account_locked() -> Self {
    Self::ClientError(ClientError::AccountLocked)
  }
}

// Implement Reply for Error, so that error messages
// are automatically created and returned on errors
impl Reply for Error {
  fn into_response(self) -> Response<Body> {
    match self {
      Self::InternalError(e) => e.into_response(),
      Self::ClientError(e) => e.into_response(),
    }
  }
}

// Implement From for library errors
// Note that the from for ParseIntError is for the most common origin
// but it can be caused by any std::parse::<integer> call, so use
// map_err with a specific utility function for other causes.
impl From<JsonError> for Error {
  fn from(e: JsonError) -> Self {
    Self::ClientError(ClientError::InvalidJson(format!("{}", e)))
  }
}
impl From<UrlEncodingError> for Error {
  fn from(e: UrlEncodingError) -> Self {
    Self::ClientError(ClientError::InvalidUrlEncoding(format!("{}", e)))
  }
}
impl From<ParseIntError> for Error {
  fn from(e: ParseIntError) -> Self {
    Self::ClientError(ClientError::InvalidIndexPath(format!("{}", e)))
  }
}
impl From<JoinError> for Error {
  fn from(e: JoinError) -> Self {
    Self::InternalError(InternalError::Join(e))
  }
}
impl From<AcquireError> for Error {
  fn from(e: AcquireError) -> Self {
    Self::InternalError(InternalError::Semaphore(e))
  }
}
impl From<HashingError> for Error {
  fn from(e: HashingError) -> Self {
    Self::InternalError(InternalError::Hash(e))
  }
}
impl From<DbError> for Error {
  fn from(e: DbError) -> Self {
    Self::InternalError(InternalError::Db(e))
  }
}
impl From<ConnectionError> for Error {
  fn from(e: ConnectionError) -> Self {
    Self::InternalError(InternalError::Connection(e))
  }
}
