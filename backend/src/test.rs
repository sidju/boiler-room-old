//! Module for integration test(s)
//!
//! Intended structure is one test that begins by starting
//! the server with either normal environment based State
//! or custom state for testing.
//!
//! Within that test, after that init, all integration
//! test cases are run.
//!
//! This structure enables parallel testing of any unit
//! tests while still preventing address collisions
//! (and needless startup delay) for the integration
//! tests.
use super::*;

use hyper::{Client, Request, Response, Body, StatusCode};
use hyper::body::Buf;
use serde::de::DeserializeOwned;

const TEST_SERVER_PORT: u16 = 38080;

async fn print_json (res: &mut Response<Body>) {
  if res.status() == hyper::StatusCode::NO_CONTENT {
    println!("Empty body");
    return;
  }
  let content_type = res.headers().get("Content-Type")
    .map(|x| x.to_str().unwrap())
    .unwrap()
  ;
  assert_eq!("application/json", content_type);
  let bytes = hyper::body::to_bytes(res.body_mut())
    .await
    .unwrap()
    .to_vec()
  ;
  let s = String::from_utf8(bytes).unwrap();
  println!("{}", s);
}
async fn from_json <T: DeserializeOwned> (res: &mut Response<Body>) -> T {
  let content_type = res.headers().get("Content-Type")
    .map(|x| x.to_str().unwrap())
    .unwrap()
  ;
  assert_eq!("application/json", content_type);
  let data: T = serde_json::from_reader(
    hyper::body::aggregate(res.body_mut())
      .await
      .unwrap()
      .reader()
  ).unwrap();
  data
}

#[tokio::test]
async fn integration_tests() {
  // Start a server for testing
  // Beware that no error is returned if the server doesn't start
  let state = init_state().await;
  let addr = SocketAddr::from(([127,0,0,1], TEST_SERVER_PORT));
  let _server = tokio::task::spawn( async move { run_server(state, addr).await; } );

  // Create a HTTP client for the testing
  let client = Client::builder()
    .http1_preserve_header_case(true)
    .build_http::<Body>()
  ;

  // Prepare the database with testing data
  let testing_password = nanoid::nanoid!(32);
  let testing_hash = crate::auth::hash::hash(
    &state.cpu_semaphore,
    &state.hasher,
    testing_password.clone(),
  ).await.unwrap();
  sqlx::query!(
    "UPDATE users SET username = 'test-admin', pass = $1, admin = true, locked = false WHERE id = -1",
    &testing_hash,
  )
    .execute(&state.db_pool)
    .await
    .unwrap()
  ;
  sqlx::query!(
    "UPDATE users SET username = 'test-user', pass = $1, admin = false, locked = false WHERE id = -2",
    &testing_hash,
  )
    .execute(&state.db_pool)
    .await
    .unwrap()
  ;


  println!("\nTest login with valid credentials.");
  // User
  let request = Request::post( format!("http://127.0.0.1:{}/api/login", TEST_SERVER_PORT) )
    .header("Content-Type", "application/json")
    .body(format!("{{ \"username\":\"test-user\", \"password\":\"{}\", \"extended\":true }}", testing_password).into())
    .unwrap()
  ;
  println!("Request to user login: {:?}", &request);
  let mut response = client.request(request).await.unwrap();
  println!("Response to user login: {:?}", response);
  assert_eq!(StatusCode::CREATED, response.status());
  let user_session: crate::auth::Session = from_json(&mut response).await;
  println!("{:?}", user_session);
  // Admin
  let request = Request::post( format!("http://127.0.0.1:{}/api/login", TEST_SERVER_PORT) )
    .header("Content-Type", "application/json")
    .body(format!("{{ \"username\":\"test-admin\", \"password\":\"{}\", \"extended\":true }}", testing_password).into())
    .unwrap()
  ;
  println!("Request to admin login: {:?}", &request);
  let mut response = client.request(request).await.unwrap();
  println!("Response to admin login: {:?}", response);
  assert_eq!(StatusCode::CREATED, response.status());
  let admin_session: crate::auth::Session = from_json(&mut response).await;
  println!("{:?}", &admin_session);


  println!("\nTest login with invalid credentials.");
  // User
  let request = Request::post( format!("http://127.0.0.1:{}/api/login", TEST_SERVER_PORT) )
    .header("Content-Type", "application/json")
    .body(format!("{{ \"username\":\"test-user\", \"password\":\"{}bad\", \"extended\":true }}", testing_password).into())
    .unwrap()
  ;
  let mut response = client.request(request).await.unwrap();
  println!("Response to user login: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  // Admin
  let request = Request::post( format!("http://127.0.0.1:{}/api/login", TEST_SERVER_PORT) )
    .header("Content-Type", "application/json")
    .body(format!("{{ \"username\":\"test-admin\", \"password\":\"{}bad\", \"extended\":true }}", testing_password).into())
    .unwrap()
  ;
  let mut response = client.request(request).await.unwrap();
  println!("Response to admin login: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());



  println!("\nTest logout with valid session.");
  // User
  let request = Request::post( format!("http://127.0.0.1:{}/api/logout", TEST_SERVER_PORT) )
    .header("Authorization", format!("bearer {}", user_session.key))
    .body("".into())
    .unwrap()
  ;
  println!("Request: {:?}", &request);
  let mut response = client.request(request).await.unwrap();
  println!("Response to user logout: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::NO_CONTENT, response.status());
  // Admin
  let request = Request::post( format!("http://127.0.0.1:{}/api/logout", TEST_SERVER_PORT) )
    .header("Authorization", format!("bearer {}", admin_session.key))
    .body("".into())
    .unwrap()
  ;
  let mut response = client.request(request).await.unwrap();
  println!("Response to admin logout: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::NO_CONTENT, response.status());


  println!("\nTest logout with invalid session.");
  // User
  let request = Request::post( format!("http://127.0.0.1:{}/api/logout", TEST_SERVER_PORT) )
    .header("Authorization", format!("bearer {}", user_session.key))
    .body("".into())
    .unwrap()
  ;
  let mut response = client.request(request).await.unwrap();
  println!("Response to user logout: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  // Admin
  let request = Request::post( format!("http://127.0.0.1:{}/api/logout", TEST_SERVER_PORT) )
    .header("Authorization", format!("bearer {}", admin_session.key))
    .body("".into())
    .unwrap()
  ;
  let mut response = client.request(request).await.unwrap();
  println!("Response to admin logout: {:?}", response);
  print_json(&mut response).await;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());


  // Cleanup database after testing
  sqlx::query!(
    "UPDATE users SET pass = NULL WHERE id = -1 OR id = -2"
  )
    .execute(&state.db_pool)
    .await
    .unwrap()
  ;
}
