use sqlx::postgres::PgPool;

use crate::Error;

// Declare a variant sqlx macro for ORDER BY
/// Generates a match over $matchee, where each branch contains a full query execution.
/// For each match a different middle is inserted between qhead and qtail, but arguments unaffected.
/// (The middle is mostly for ORDER BY and the tail is usually a secondary ORDER BY or LIMIT).
#[macro_export]
macro_rules! sqlx_order {
  ( $ret_struct:path, $db:expr ; $qhead:literal, $qtail:literal $(, $argument:expr)* $(,)? ; $matchee:expr $(; $pattern:pat , $middle:literal)* $(;)? ) => {
    sqlx_order!( @bracketed $ret_struct, $db; $qhead, $qtail, [$( $argument ),*] ; $matchee $(; $pattern, $middle )* )
  };
  ( @bracketed $ret_struct:path, $db:expr ; $qhead:literal, $qtail:literal, $argument:tt ; $matchee:expr $(; $pattern:pat , $middle:literal)* $(;)? ) => {
    match $matchee {
      $(
      $pattern => {
        sqlx_order!( @inner $ret_struct, $db; $qhead + $middle + $qtail, $argument )        
      }
      )*
    }
  };
  ( @inner $ret_struct:path, $db:expr ; $query:expr, [ $( $argument:expr),* ] ) => {
    sqlx::query_as!( $ret_struct, $query, $( $argument ),* )
      .fetch_all($db)
      .await?
  };
}

pub async fn update_admin(
  db_pool: &PgPool,
  hash: String,
) -> Result<(), Error> {
  sqlx::query!(
    "UPDATE users SET username = 'admin', pass = $1, admin = true, locked = false WHERE id = 0",
    hash,
  )
    .execute(db_pool)
    .await?
  ;
  Ok(())
}
