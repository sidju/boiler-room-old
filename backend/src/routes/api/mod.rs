use super::*;
use shared_types::Login;

mod user;
mod admin;

pub async fn route(
  state: &'static State,
  mut req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      Ok(Response::new(include_str!("doc_body.txt").into()))
    },
    Some("login") => {
      verify_method_path_end(&path_vec, &req, &Method::POST)?;
      // Parse out request
      let credentials: Login = parse_json(&mut req, state.max_content_len).await?;
      // Call login handler
      let session = crate::auth::login(state, credentials).await?;
      // Slightly wrap up the result
      match session {
        Some(session) => {
          json(&session)
            .map(|mut re| {
              *re.status_mut() = StatusCode::CREATED;
              re
            })
        },
        None => Err(Error::bad_login()),
      }
    },
    Some("admin") => {
      // Require authentication
      let session_key = unwrap_bearer(get_header(&req, "Authorization")?);
      let permissions = crate::auth::require_admin(
        state,
        session_key.clone(),
      ).await?;
      // Call into detail routing
      admin::route(
        state,
        req,
        path_vec,
        permissions,
      ).await
    },
    Some(p) => {
      // Require authentication
      let session_key = unwrap_bearer(get_header(&req, "Authorization")?);
      let permissions = crate::auth::require_session(
        state,
        session_key.clone(),
      ).await?;
      match p {
        "logout" => {
          verify_method_path_end(&path_vec, &req, &Method::POST)?;
          // Call logout handler
          crate::auth::logout(state, session_key).await?;
          empty()
        },
        "user" => {
          user::route(
            state,
            req,
            path_vec,
            permissions,
          ).await
        },
        _ => {
          Err(Error::path_not_found(&req))
        },
      }
    },
  }
}
