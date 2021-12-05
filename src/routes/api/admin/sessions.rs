use super::*;

#[derive(Serialize)]
struct ReturnableSession {
  id: i32,
  userid: i32,
  until: NaiveDateTime,
}

#[derive(Deserialize)]
enum SessionsOrder {
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
impl Default for SessionsOrder {
  fn default() -> Self { Self::UntilAsc }
}
#[derive(Deserialize)]
struct SessionsFilter {
  id_mte: Option<i32>,
  id_lte: Option<i32>,
  userid_eq: Option<i32>,
  until_lte: Option<NaiveDateTime>,
  until_mte: Option<NaiveDateTime>,
  #[serde(default)]
  order_by: SessionsOrder,
  limit: Option<i64>,
}

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
  _permissions: Permissions
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    // In base path get a list of the IDs the user can access
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Parse out query part of URI into filter
      let filter: SessionsFilter = parse_filter(&req)?;
      // Fetch the data from database
      // Note the null checking around every filter
      let sessions = sqlx_order!( ReturnableSession, &state.db_pool;
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
        SessionsOrder::IdAsc , "ORDER BY id ASC";
        SessionsOrder::IdDesc , "ORDER BY id DESC";
        SessionsOrder::UserIdAsc , "ORDER BY userid ASC";
        SessionsOrder::UserIdDesc , "ORDER BY userid DESC";
        SessionsOrder::UntilAsc , "ORDER BY until ASC";
        SessionsOrder::UntilDesc , "ORDER BY until DESC";
      );
      if sessions.is_empty() {
        empty()
      } else {
        json(&sessions)
      }
    },
    // If there is more than base path, parse it to a session ID and get it
    // (of course filtering to IDs owned by user.
    Some(sessionid) => {
      verify_method_path_end(&path_vec, &req, &Method::DELETE)?;
      let parsed = sessionid.parse::<i32>()?;
      let affected = sqlx::query!(
        "DELETE FROM sessions WHERE id = $1",
        parsed
      )
        .execute(&state.db_pool)
        .await
        .map_err(Error::from)
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
  }
}
