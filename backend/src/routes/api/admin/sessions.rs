use super::*;

use shared_types::AdminReturnableSession;
use shared_types::{AdminSessionsFilter, AdminSessionsOrder};

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Parse out query part of URI into filter
      let filter: AdminSessionsFilter = parse_filter(&req)?;
      // Fetch the data from database
      // Note the null checking around every filter
      let sessions = sqlx_order!( AdminReturnableSession, &state.db_pool;
        "
SELECT id, userid, until FROM sessions
WHERE
  id <= $1 OR $1 IS NULL AND
  id >= $2 OR $2 IS NULL AND
  userid = $3 OR $3 IS NULL AND
  until <= $4 OR $4 IS NULL AND
  until <= $5 OR $5 IS NULL AND
  until >= NOW()
        ",
        "
LIMIT $6
        ",
        filter.id_lte,
        filter.id_mte,
        filter.userid_eq,
        filter.until_lte,
        filter.until_mte,
        filter.limit,
        // Define match cases and what ORDER TO to insert for each
        ; filter.order_by ;
        AdminSessionsOrder::IdAsc , "ORDER BY id ASC";
        AdminSessionsOrder::IdDesc , "ORDER BY id DESC";
        AdminSessionsOrder::UserIdAsc , "ORDER BY userid ASC";
        AdminSessionsOrder::UserIdDesc , "ORDER BY userid DESC";
        AdminSessionsOrder::UntilAsc , "ORDER BY until ASC";
        AdminSessionsOrder::UntilDesc , "ORDER BY until DESC";
      );
      if sessions.is_empty() {
        empty()
      } else {
        json(&sessions)
      }
    }
    // If there is more than base path, parse it to a session ID and get it
    Some(sessionid) => {
      verify_method_path_end(&path_vec, &req, &Method::DELETE)?;
      let parsed = sessionid.parse::<i32>()?;
      let affected = sqlx::query!("DELETE FROM sessions WHERE id = $1", parsed)
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
