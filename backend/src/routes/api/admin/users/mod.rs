use super::*;

mod user;

use shared_types::{AdminReturnableUser, NewUser, UsersFilter, UsersOrder};

pub async fn route(
  state: &'static State,
  mut req: Request,
  mut path_vec: Vec<String>,
  permissions: Permissions,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => {
          // Parse out query part of URI into filter
          let filter: UsersFilter = parse_filter(&req)?;
          // Fetch the data from database
          // Note the null checking around every filter
          let users = sqlx_order!( AdminReturnableUser, &state.db_pool;
            "
SELECT id, username, admin, locked FROM users
WHERE
      id <= $1 OR $1 IS NULL AND
      id >= $2 OR $2 IS NULL AND
      username ~ $3 OR $3 IS NULL AND
      username !~ $4 OR $4 IS NULL AND
      admin = $5 OR $5 IS NULL AND
      locked = $6 OR $6 IS NULL
            ",
            "
LIMIT $7
            ",
            filter.id_lte,
            filter.id_mte,
            filter.username_regex,
            filter.username_nregex,
            filter.admin_eq,
            filter.locked_eq,
            filter.limit,
            // Define match cases and what ORDER TO to insert for each
            ; filter.order_by ;
            UsersOrder::IdAsc , "ORDER BY id ASC";
            UsersOrder::IdDesc , "ORDER BY id DESC";
            UsersOrder::UsernameAsc , "ORDER BY username ASC";
            UsersOrder::UsernameDesc , "ORDER BY username DESC";
          );
          if users.is_empty() {
            empty()
          } else {
            json(&users)
          }
        }
        &Method::POST => {
          let new_user: NewUser = parse_json(&mut req, state.max_content_len).await?;
          let created_user = sqlx::query_as!(
            AdminReturnableUser,
            "
INSERT INTO users(username,locked,admin) VALUES($1,$2,$3)
RETURNING id,username,admin,locked
            ",
            new_user.username,
            new_user.locked,
            new_user.admin,
          )
          .fetch_one(&state.db_pool)
          .await
          .map_err(|e| -> Error {
            match e {
              sqlx::Error::Database(ref err) => match err.constraint() {
                Some("username") => Error::username_taken(),
                _ => e.into(),
              },
              _ => e.into(),
            }
          })?;
          set_status(json(&created_user), StatusCode::CREATED)
        }
        _ => Err(Error::method_not_found(&req)),
      }
    }
    // If there is more than base path parse it as a userid
    Some(sessionid) => {
      let parsed = sessionid.parse::<i32>()?;
      user::route(state, req, path_vec, permissions, parsed).await
    }
  }
}
