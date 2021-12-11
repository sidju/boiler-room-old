use super::*;

mod users;
mod sessions;

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
  permissions: Permissions,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None | Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      Ok(Response::new(include_str!("doc_body.txt").into()))
    },
    Some("users") => {
      users::route(
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
      ).await
    },
    Some(_) => Err(Error::PathNotFound(
      format!("{}", req.uri().path())
    )),
  }
}
