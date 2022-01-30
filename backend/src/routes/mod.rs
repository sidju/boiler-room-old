use hyper::{Method, StatusCode};
use hyper::header::HeaderValue;
use std::convert::Infallible;

use crate::auth::Permissions;
use crate::sqlx_order;
use crate::{Error, Reply, State};

mod utils;
pub use utils::*;

mod api;

type Response = hyper::Response<hyper::Body>;
type Request = hyper::Request<hyper::Body>;

pub async fn handle_request(state: &'static State, req: Request) -> Result<Response, Infallible> {
  // Call inner, unwrap its opaque type to a Response<Body>
  Ok(route(state, req).await.into_response())
}

// Static files for the frontend
static INDEX_HTML: &str = include_str!("../../../frontend/index.html");
static INDEX_HTML_ETAG: HeaderValue = HeaderValue::from_static(include_str!("../../../frontend/index.html.etag"));
static PACKAGE_JS: &str = include_str!("../../../frontend/pkg/package.js");
static PACKAGE_JS_ETAG: HeaderValue = HeaderValue::from_static(include_str!("../../../frontend/pkg/package.js.etag"));
static PACKAGE_WASM: &[u8] = include_bytes!("../../../frontend/pkg/package_bg.wasm");
static PACKAGE_WASM_ETAG: HeaderValue = HeaderValue::from_static(include_str!("../../../frontend/pkg/package_bg.wasm.etag"));
// Client-cache config for them
// Cache for one week, use and validate in the last day of that week.
// Daily and consistent weekly users will always use cache, upgrades can be
// done by setting the header below to 'no-cache' (aka. always validate) 
static CACHE_CONTROL: HeaderValue = HeaderValue::from_static(
  //"max-age=518400,stale-while-revalidate=172800" // Recommended value for production use
  // For upgrades, set this 1 week and 1 day before deployment
  // (so all client caches have been stale once)
  "max-age=0,must-revalidate" // Recommended value for development
);

// We have an inner handler to allow returning our own types,
// converting them into responses in the outer handler
async fn route(state: &'static State, req: Request) -> Result<Response, Error> {
  // Match on path to send into API or HTML module
  let mut path_vec: Vec<String> = req
    .uri()
    .path()
    .split('/')
    .rev()
    .map(|s| s.to_owned())
    .collect();

  // If this path is functional the first
  // part will be None or Some(""), so error on else
  match path_vec.pop().as_deref() {
    None | Some("") => (),
    Some(unexpected) => {
      return Err(Error::path_data_before_root(unexpected.to_string()));
    }
  };

  match path_vec.pop().as_deref() {
    Some("api") => {
      let mut re = api::route(state, req, path_vec).await?;
      // Tell clients not to cache API responses
      re.headers_mut().insert("cache-control", HeaderValue::from_static("no-store"));
      Ok(re)
    },
    // Serve webassembly file
    Some("package.js") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Use if-none-match to only send data if needed
      let etag = req.headers().get("if-none-match");
      let mut re = if Some(&PACKAGE_JS_ETAG) == etag {
        not_modified()?
      } else {
        javascript(PACKAGE_JS)?
      };
      re.headers_mut().insert("etag", PACKAGE_JS_ETAG.clone());
      re.headers_mut().insert("cache-control", CACHE_CONTROL.clone());
      Ok(re)
    }
    Some("package_bg.wasm") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Use if-none-match to only send data if needed
      let etag = req.headers().get("if-none-match");
      let mut re = if Some(&PACKAGE_WASM_ETAG) == etag {
        not_modified()?
      } else {
        webassembly(PACKAGE_WASM)?
      };
      re.headers_mut().insert("etag", PACKAGE_WASM_ETAG.clone());
      re.headers_mut().insert("cache-control", CACHE_CONTROL.clone());
      Ok(re)
    }
    // For ALL other paths, serve index file
    _ => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Use if-none-match to only send data if needed
      let etag = req.headers().get("if-none-match");
      let mut re = if Some(&INDEX_HTML_ETAG) == etag {
        not_modified()?
      } else {
        html(INDEX_HTML)?
      };
      re.headers_mut().insert("etag", INDEX_HTML_ETAG.clone());
      re.headers_mut().insert("cache-control", CACHE_CONTROL.clone());
      Ok(re)
    }
  }
}
