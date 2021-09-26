use super::*;
use crate::auth::Login;

pub async fn route(
  state: &'static State,
  mut req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      Ok(Response::new("\
API:
  login:
    Takes a form containing username(string), password(string) and extended(bool as string, 'true' or 'false').
    The extended flag defines wether the session is valid for 1 day (if false) or 1 year (if true).
    If successful returns session data as a json body, containing key(string) and time of expiry.
  logout:
    Requires valid session.
    Takes any post (data ignored) and deletes the session used to access the handler.
    If successful returns nothing (HTTP status 204).
Errors:
todo\
      ".into()))
    },
    Some("login") => {
      verify_method_path_end(&path_vec, &req, &Method::POST)?;
      // Parse out request
      let form: Login = from_body(&mut req).await?;
      // Call login handler
      let session = crate::auth::login(state, form).await?;
      // Slightly wrap up the result
      match session {
        Some(session) => {
          json(&session)
            .map(|mut re| {
              *re.status_mut() = StatusCode::CREATED;
              re
            })
        },
        None => Err(Error::BadLogin),
      }
    },
    Some("logout") => {
      verify_method_path_end(&path_vec, &req, &Method::POST)?;
      // Require authentication
      let session_key = crate::auth::unwrap_bearer(&get_header(&req, "Authorization")?);
      crate::auth::require_session(
        state,
        session_key.clone(),
      ).await?;
      // Call logout handler
      crate::auth::logout(state, session_key).await?;
      // Create simple response
      let mut re = Response::new("".into());
      *re.status_mut() = StatusCode::NO_CONTENT;
      Ok(re)
    },
    Some(_) => {
      Err(Error::PathNotFound( format!("{}", req.uri().path()) ))
    },
  }
}
