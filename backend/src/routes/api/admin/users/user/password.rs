use super::*;

use shared_types::PasswordReset;

pub async fn route(
  state: &'static State,
  mut req: Request,
  path_vec: Vec<String>,
  permissions: Permissions,
  userid: i32,
) -> Result<Response, Error> {
  verify_path_end(&path_vec, &req)?;
  match req.method() {
    &Method::DELETE => {
      if userid < 1 { return Err(Error::method_not_found(&req)); }
      sqlx::query!(
        "UPDATE users SET pass = NULL WHERE id = $1",
        userid,
      )
        .execute(&state.db_pool)
        .await?
      ;
      // Also invalidate sessions, as per documentation
      sqlx::query!(
        "DELETE FROM sessions WHERE userid = $1",
        userid,
      )
        .execute(&state.db_pool)
        .await?
      ;
      empty()
    },
    &Method::POST => {
      let query: PasswordReset = parse_json(&mut req, state.max_content_len).await?;
    
      // Verify the admin_password, so it takes more than a session key to
      // create unlimited session keys
      let admin_user = sqlx::query!(
        "SELECT pass, locked FROM users WHERE id = $1",
        permissions.userid
      )
        .fetch_one(&state.db_pool)
        .await?
      ;
      let admin_hash = match admin_user.pass {
        Some(hash) => {
          hash
        },
        None => {
          // Normally impossible, since setting passhash to None
          // also deletes all sessions (but maybe race condition).
          // However, impersonate makes it possible again.
          return Err(Error::bad_login());
        },
      };
      let correct_pass = crate::auth::hash::verify(
        &state.cpu_semaphore,
        &state.hasher,
        admin_hash,
        query.admin_password,
      )
        .await?
      ;
      if !correct_pass { return Err(Error::bad_login()); }
      if admin_user.locked { return Err(Error::account_locked()); }

      // Hash the new user password
      let new_hash = crate::auth::hash::hash(
        &state.cpu_semaphore,
        &state.hasher,
        query.new_password,
      )
        .await?
      ;
      // Apply the new password
      sqlx::query!(
        "UPDATE users SET pass = $1 WHERE id = $2",
        new_hash,
        userid,
      )
        .execute(&state.db_pool)
        .await?
      ;
      // If clear_sessions given we do so _after_ changing the password
      if query.clear_sessions {
        sqlx::query!(
          "DELETE FROM sessions WHERE userid = $1",
          userid
        )
          .execute(&state.db_pool)
          .await?
        ;
      }
      empty()
    },
    _ => Err(Error::method_not_found(&req)),
  }
}
