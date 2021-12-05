use super::*;

#[derive(Serialize)]
pub struct ReturnableUser {
  id: i32,
  username: String,
  admin: bool
}

#[derive(Deserialize)]
struct PasswordChange {
  admin_password: String,
  new_password: String,
  clear_sessions: bool,
}
