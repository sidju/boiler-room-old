use super::*;

use shared_types::ReturnableSession;
use shared_types::{SessionsFilter, SessionsOrder};

pub async fn route(
    state: &'static State,
    req: Request,
    mut path_vec: Vec<String>,
    permissions: Permissions,
) -> Result<Response, Error> {
    match path_vec.pop().as_deref() {
        // In base path, list user's sessions with filtering
        None | Some("") => {
            verify_method_path_end(&path_vec, &req, &Method::GET)?;
            // Parse out query part of URI into filter
            let filter: SessionsFilter = parse_filter(&req)?;
            // Fetch the data from database
            // Note the null checking around every filter
            let sessions = sqlx_order!( ReturnableSession, &state.db_pool;
              "
SELECT id, until FROM sessions
WHERE
  id <= $1 OR $1 IS NULL AND
  id >= $2 OR $2 IS NULL AND
  until <= $3 OR $3 IS NULL AND
  until <= $4 OR $4 IS NULL AND
  until >= NOW() AND
  userid = $5
        ",
              "
LIMIT $6
        ",
              filter.id_lte,
              filter.id_mte,
              filter.until_lte,
              filter.until_mte,
              permissions.userid,
              filter.limit,
              // Define match cases and what ORDER TO to insert for each
              ; filter.order_by ;
              SessionsOrder::IdAsc , "ORDER BY id ASC";
              SessionsOrder::IdDesc , "ORDER BY id DESC";
              SessionsOrder::UntilAsc , "ORDER BY until ASC";
              SessionsOrder::UntilDesc , "ORDER BY until DESC";
            );
            if sessions.is_empty() {
                empty()
            } else {
                json(&sessions)
            }
        }
        // If there is more than base path, parse it to a session ID and get it
        // (of course filtering to IDs owned by user.
        Some(sessionid) => {
            verify_method_path_end(&path_vec, &req, &Method::DELETE)?;
            let parsed = sessionid.parse::<i32>()?;
            let affected = sqlx::query!(
                "DELETE FROM sessions WHERE userid = $1 AND id = $2",
                permissions.userid,
                parsed
            )
            .execute(&state.db_pool)
            .await?
            .rows_affected();
            match affected {
                0 => Err(Error::path_not_found(&req)),
                _ => empty(),
            }
        }
    }
}
