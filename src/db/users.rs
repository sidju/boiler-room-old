// Methods for interacting based on users.id
use super::*;

pub struct User {
  email: String,
  pass: String,
  admin: bool,
}

pub async fn update_admin(
  db_pool: &PgPool,
  hash: String,
) -> Result<(), Error> {
  sqlx::query!(
    "
INSERT INTO users(id, email, pass, admin) VALUES(0,'admin',$1,'true')
ON CONFLICT (id) DO UPDATE SET email = 'admin', pass = $1, admin = true
    ",
    hash,
  )
    .execute(db_pool)
    .await
    .map_err(|e| Error::Db(e))
    .map(|_| ()) // Drop the number of rows affected
}

pub async fn add_user(
  db_pool: &PgPool,
  user: User,
) -> Result<(), Error> {
  sqlx::query!(
    "INSERT INTO users(email, pass, admin) VALUES($1,$2,$3)",
    user.email,
    user.pass,
    user.admin,
  )
    .execute(db_pool)
    .await
    .map_err(|e| Error::Db(e))
    .map(|_| ()) // Drop the number of rows affected
}

