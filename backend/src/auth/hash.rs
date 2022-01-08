//! A module for password hashing and session validation.
//!
//! Wraps the argon2 crate with async and
//! provides implementations of warp::Filter
//! that extract and validate sessions

use tokio::sync::Semaphore;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, PasswordHash};
use argon2::password_hash::SaltString;
use rand_core::OsRng;

use crate::Error;

pub async fn hash(
  cpu_semaphore: &Semaphore,
  hasher: &Argon2<'static>,
  password: String,
) -> Result<String, Error>{
  // Before creating a blocking thread, get a slot from the CPU-bound semaphore
  // The semaphore is used globally to prevent saturating the tokio thread-pool
  let _handle = cpu_semaphore.acquire().await;

  // Then we clone the hasher, since it is cheap to clone and solves
  // the static lifetime requirement of spawn_blocking
  let hasher = hasher.clone();
  // If the password wasn't owned we'd need to clone that as well

  Ok(
    tokio::task::spawn_blocking(move || {
      // We generate salt from OsRng
      let salt = SaltString::generate(&mut OsRng);
      // Then hash the password using the given hasher and generated salt
      hasher.hash_password_simple(&password.as_bytes(), &salt)
      // Finally we return the passhash after converting into string
        .map(|hash| hash.to_string())
    }).await
      ? // To unwrap the outer layer of this Result<Result<>>
      ?
  )
}

pub async fn verify(
  cpu_semaphore: &Semaphore,
  hasher: &Argon2<'static>,
  hash: String,
  password: String,
) -> Result<bool, Error>{
  // Before creating a blocking thread, get a slot from the CPU-bound semaphore
  // The semaphore is used globally to prevent saturating the tokio thread-pool
  let _handle = cpu_semaphore.acquire().await;

  // Then we clone the hasher, since it is cheap to clone and solves
  // the static lifetime requirement of spawn_blocking
  let hasher = hasher.clone();
  // If the password wasn't owned we'd need to clone that as well

  match tokio::task::spawn_blocking(move || {
    // Parse the hash into a struct which provides the configuration and salt
    // to hash the password identically
    let hash = PasswordHash::new(&hash)?;
    // Then hash the password using the given hasher and generated salt
    hasher.verify_password(&password.as_bytes(), &hash)
  }).await
    ? // To unwrap the outer layer of this Result<Result<>>
  {
    Ok(()) => Ok(true),
    Err(argon2::password_hash::Error::Password) => Ok(false),
    Err(e) => Err(Error::from(e)),
  }
}
