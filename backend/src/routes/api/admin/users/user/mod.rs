use super::*;

mod impersonate;
mod password;

use shared_types::UpdateUser;

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
                    let user = sqlx::query_as!(
                        super::AdminReturnableUser,
                        "SELECT id, username, admin, locked FROM users WHERE id = $1",
                        userid
                    )
                    .fetch_one(&state.db_pool)
                    .await?;
                    json(&user)
                }
                &Method::PUT => {
                    if userid < 1 {
                        return Err(Error::method_not_found(&req));
                    }
                    let update: UpdateUser = parse_json(&mut req, state.max_content_len).await?;
                    let updated = sqlx::query_as!(
                        super::AdminReturnableUser,
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
                    .await?;
                    json(&updated)
                }
                &Method::DELETE => {
                    if userid < 1 {
                        return Err(Error::method_not_found(&req));
                    }
                    let affected = sqlx::query!("DELETE FROM users WHERE id = $1", userid)
                        .execute(&state.db_pool)
                        .await?
                        .rows_affected();
                    match affected {
                        0 => Err(Error::path_not_found(&req)),
                        _ => empty(),
                    }
                }
                _ => Err(Error::method_not_found(&req)),
            }
        }
        Some("password") => password::route(state, req, path_vec, permissions, userid).await,
        Some("impersonate") => impersonate::route(state, req, path_vec, permissions, userid).await,
        _ => Err(Error::path_not_found(&req)),
    }
}
