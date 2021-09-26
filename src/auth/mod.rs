//! A module for password hashing and session validation.
//!
//! Wraps the argon2 crate with async and
//! provides implementations of warp::Filter
//! that extract and validate sessions

pub mod hash;
pub mod session;
pub use session::*;
pub mod login;
pub use login::*;

// The struct given to each handler
// It should contain everything needed to know
// the user and its permissions
#[derive(serde::Serialize, Debug)]
pub struct Permissions {
  // For use by html rendering handlers to print in top-bar
  pub username: String,
  // To identify if the current user owns a resource
  pub userid: i32,
  // To identify if the current user has admin perms
  pub admin: bool,
}
