use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// Login form struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Login {
  pub username: String,
  pub password: String,
  pub extended: bool, // If true we make session last longer
}

// Session struct, describing created Session
// Since it allows impersonation this is only given out at login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
  pub id: i32,
  pub key: String,
  pub until: NaiveDateTime,
}

// Version of session that can be returned to user without
// allowing impersonation (or handing out the implied userid)
#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnableSession {
  pub id: i32,
  pub until: NaiveDateTime,
}
// Types to allow filtering over user's own sessions
#[derive(Debug, Serialize, Deserialize)]
pub enum SessionsOrder {
  #[serde(alias = "id_asc")]
  IdAsc,
  #[serde(alias = "id_desc")]
  IdDesc,
  #[serde(alias = "until_asc")]
  UntilAsc,
  #[serde(alias = "until_desc")]
  UntilDesc,
}
impl Default for SessionsOrder {
  fn default() -> Self {
    Self::UntilAsc
  }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionsFilter {
  pub id_mte: Option<i32>,
  pub id_lte: Option<i32>,
  pub until_lte: Option<NaiveDateTime>,
  pub until_mte: Option<NaiveDateTime>,
  #[serde(default)]
  pub order_by: SessionsOrder,
  pub limit: Option<i64>,
}

// The same structs for admin, when userid is not implied
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminReturnableSession {
  pub id: i32,
  pub userid: i32,
  pub until: NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum AdminSessionsOrder {
  #[serde(alias = "id_asc")]
  IdAsc,
  #[serde(alias = "id_desc")]
  IdDesc,
  #[serde(alias = "userid_asc")]
  UserIdAsc,
  #[serde(alias = "userid_desc")]
  UserIdDesc,
  #[serde(alias = "until_asc")]
  UntilAsc,
  #[serde(alias = "until_desc")]
  UntilDesc,
}
impl Default for AdminSessionsOrder {
  fn default() -> Self {
    Self::UntilAsc
  }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminSessionsFilter {
  pub id_mte: Option<i32>,
  pub id_lte: Option<i32>,
  pub userid_eq: Option<i32>,
  pub until_lte: Option<NaiveDateTime>,
  pub until_mte: Option<NaiveDateTime>,
  #[serde(default)]
  pub order_by: AdminSessionsOrder,
  pub limit: Option<i64>,
}

// Cleaned up user for returning to users
#[derive(Debug, Serialize, Deserialize)]
pub struct ReturnableUser {
  pub id: i32,
  pub username: String,
  pub admin: bool,
}
// Same with some additional admin-only data
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminReturnableUser {
  pub id: i32,
  pub username: String,
  pub admin: bool,
  pub locked: bool,
}
// Types to filter user lookups (admin only)
#[derive(Debug, Serialize, Deserialize)]
pub enum UsersOrder {
  #[serde(alias = "id_asc")]
  IdAsc,
  #[serde(alias = "id_desc")]
  IdDesc,
  #[serde(alias = "username_asc")]
  UsernameAsc,
  #[serde(alias = "username_desc")]
  UsernameDesc,
}
impl Default for UsersOrder {
  fn default() -> Self {
    Self::UsernameAsc
  }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UsersFilter {
  pub id_mte: Option<i32>,
  pub id_lte: Option<i32>,
  pub username_regex: Option<String>,
  pub username_nregex: Option<String>,
  pub admin_eq: Option<bool>,
  pub locked_eq: Option<bool>,
  #[serde(default)]
  pub order_by: UsersOrder,
  pub limit: Option<i64>,
}

// Form struct for users changing their password
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordChange {
  pub old_password: String,
  pub new_password: String,
  pub clear_sessions: bool,
}

// User administration forms
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
  pub username: String,
  #[serde(default)]
  pub admin: bool,
  #[serde(default)]
  pub locked: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
  pub username: String,
  pub admin: bool,
  pub locked: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Impersonate {
  pub admin_password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordReset {
  pub admin_password: String,
  pub new_password: String,
  pub clear_sessions: bool,
}

// Declare an object for public errors
// These are fully returned as json to API users
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientError {
  // We start with simple routing errors
  PathNotFound(String),
  MethodNotFound(String),
  Unauthorized,
  Forbidden,

  // Then parsing errors
  PathDataBeforeRoot(String),
  UnreadableHeader(String),
  InvalidContentLength(String),
  InvalidContentType(String),
  InvalidJson(String),
  InvalidUrlEncoding(String),
  InvalidIndexPath(String),

  // Finally non-parsing user errors
  BadPassword,
  UsernameTaken,
  BadLogin,
  AccountLocked,
}
