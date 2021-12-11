use super::*;

#[derive(Deserialize)]
struct PasswordChange {
  old_password: String,
  new_password: String,
  clear_sessions: bool,
}

pub async fn route(
  state: &'static State,
  mut req: Request,
  path_vec: Vec<String>,
  permissions: Permissions,
) -> Result<Response, Error> {
  verify_method_path_end(&path_vec, &req, &Method::POST)?;
  // Parse out request
  let password_change: PasswordChange = parse_json(&mut req).await?;
  // Verify current session via password in password_change
  let user_hash = match sqlx::query!(
    "SELECT pass FROM users WHERE id = $1",
    permissions.userid
  )
    .fetch_one(&state.db_pool)
    .await?
    .pass
  {
    Some(pass) => pass,
    None => { return Err(Error::Unauthorized); },
  };
  if ! crate::auth::hash::verify(
    &state.cpu_semaphore,
    &state.hasher,
    user_hash,
    password_change.old_password,
  )
    .await?
  { return Err(Error::Unauthorized); }
  // When the user has been verified, apply the password change
  let new_hash = crate::auth::hash::hash(
    &state.cpu_semaphore,
    &state.hasher,
    password_change.new_password,
  )
    .await?
  ;
  sqlx::query!(
    "UPDATE users SET pass = $1 WHERE id = $2",
    new_hash,
    permissions.userid,
  )
    .execute(&state.db_pool)
    .await?
  ;
  // Clear sessions, if requested
  if password_change.clear_sessions {
    sqlx::query!("DELETE FROM sessions WHERE userid = $1", permissions.userid)
      .execute(&state.db_pool)
      .await?
    ;
  }
  empty()
}
