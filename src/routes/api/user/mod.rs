use super::*;

mod password;
mod sessions;

#[derive(Serialize)]
pub struct ReturnableUser {
  id: i32,
  username: String,
  admin: bool
}

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
  permissions: Permissions,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Return the public information on the user
      let user = sqlx::query_as!(ReturnableUser,
        "SELECT id, username, admin FROM users WHERE id = $1",
        permissions.userid,
      )
        .fetch_one(&state.db_pool)
        .await
        ?
      ;
      json(&user)
    },
    Some("password") => {
      password::route(
        state,
        req,
        path_vec,
        permissions,
      ).await
    },
    Some("sessions") => {
      sessions::route(
        state,
        req,
        path_vec,
        permissions,
      ).await
    },
    Some(_) => Err(Error::PathNotFound(
      format!("{}", req.uri().path())
    )),
  }
}
