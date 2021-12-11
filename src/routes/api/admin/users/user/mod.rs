use super::*;

mod password;
mod impersonate;

#[derive(Deserialize)]
struct UpdateUser {
  username: String,
  admin: bool,
  locked: bool,
}

pub async fn route(
  state: &'static State,
  mut req: Request,
  mut path_vec: Vec<String>,
  permissions: Permissions,
  userid: i32,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => {
          let user = sqlx::query_as!(super::ReturnableUser,
            "SELECT id, username, admin, locked FROM users WHERE id = $1",
            userid
          )
            .fetch_one(&state.db_pool)
            .await?
          ;
          json(&user)
        },
        &Method::PUT => {
          if userid < 1 { return Err(Error::MethodNotFound(req.method().clone())); }
          let update: UpdateUser = parse_json(&mut req).await?;
          let updated = sqlx::query_as!(super::ReturnableUser,
            "
UPDATE users SET username = $2, admin = $3, locked = $4 WHERE id = $1
RETURNING id, username, admin, locked
            ",
            userid,
            update.username,
            update.admin,
            update.locked,
          )
            .fetch_one(&state.db_pool)
            .await?
          ;
          json(&updated)
        },
        &Method::DELETE => {
          if userid < 1 { return Err(Error::MethodNotFound(req.method().clone())); }
          let affected = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            userid
          )
            .execute(&state.db_pool)
            .await
            ?
            .rows_affected()
          ;
          match affected {
            0 => Err(Error::PathNotFound(
              format!("{}", req.uri().path())
            )),
            _ => empty(),
          }
        },
        _ => Err(Error::MethodNotFound(req.method().clone())),
      }
    },
    Some("password") => {
      password::route(
        state,
        req,
        path_vec,
        permissions,
        userid,
      ).await
    },
    Some("impersonate") => {
      impersonate::route(
        state,
        req,
        path_vec,
        permissions,
        userid,
      ).await
    },
    _ => Err(Error::PathNotFound(
      format!("{}", req.uri().path())
    )),
  }
}
