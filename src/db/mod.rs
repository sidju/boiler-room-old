use sqlx::postgres::PgPool;
use crate::Error;

pub async fn update_admin(
  db_pool: &PgPool,
  hash: String,
) -> Result<(), Error> {
  sqlx::query!(
    "
INSERT INTO users(id,username,pass,admin) VALUES(0,'admin',$1,'true')
ON CONFLICT (id) DO UPDATE SET username = 'admin', pass = $1, admin = true
    ",
    hash,
  )
    .execute(db_pool)
    .await
    .map_err(|e| Error::Db(e))
    .map(|_| ()) // Drop the number of rows affected
}
