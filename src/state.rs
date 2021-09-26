use tokio::sync::Semaphore;
use argon2::Argon2;
use std::env::var;

pub struct State {
  // Used to limit the number of concurrent CPU-bound tasks
  pub cpu_semaphore: Semaphore,
  // Configuration for hashing new passwords
  // Old passwords have their configuration saved in the hash
  pub hasher: Argon2<'static>,
  // Connection pool to database
  pub db_pool: sqlx::postgres::PgPool,

  // Configurations used directly
  pub login_delay: u64,
}

pub async fn init_state() -> &'static State {
  // Use dotenv to support environment file
  dotenv::dotenv().unwrap();

  // Get data from environment
  let secret_key = Box::leak(Box::new(
    var("PASSHASH_SECRET_KEY")
      .expect("PASSHASH_SECRET_KEY muste be present in environment or .env.")
  ));
  let max_nr_cpu_threads = var("MAX_NR_CPU_THREADS")
      .expect("MAX_NR_CPU_THREADS must be present in environment or .env.")
      .parse::<usize>()
      .expect("MAX_NR_CPU_THREADS could not be parsed as an unsigned integer.")
  ;
  let db_connstring = var("DATABASE_URL")
    .expect("DB_CONNSTRING must be present in environment or .env.")
  ;
  let admin_password = var("ADMIN_PASSWORD")
    .expect("ADMIN_PASSWORD must be present in environment or .env.")
  ;
  let login_delay = var("LOGIN_DELAY")
    .expect("LOGIN_DELAY must be present in environment or .env.")
    .parse::<u64>()
    .expect("LOGIN_DELAY could not be parsed as an unsigned integer.")
  ;

  // When we have all needed data, construct objects
  let cpu_semaphore = Semaphore::new(max_nr_cpu_threads);
  let hasher = Argon2::new(
    Some( secret_key.as_bytes() ),
    argon2::Params::DEFAULT_T_COST,
    argon2::Params::DEFAULT_M_COST,
    argon2::Params::DEFAULT_P_COST,
    argon2::Version::default(),
  ).unwrap();
  let db_pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(4)
    .min_connections(1)
    .idle_timeout(Some(std::time::Duration::from_secs(300)))
    .connect(&db_connstring)
    .await
    .expect("Failed to connect to database")
  ;

  // When state exists we initialize all the things related to it
  sqlx::migrate!("./migrations")
    .run(&db_pool)
    .await
    .expect("Failed to run migrations on startup.")
  ;
  println!("Migrations applied.");
  let admin_password_hash = crate::auth::hash::hash(
    &cpu_semaphore,
    &hasher,
    admin_password.clone(),
  ).await.unwrap();
  crate::db::update_admin(&db_pool, admin_password_hash).await.unwrap();
  println!("Admin password updated from .env.");

  // Return a reference to a State struct with 'static lifetime
  Box::leak( Box::new( State {
    hasher: hasher,
    cpu_semaphore: cpu_semaphore,
    db_pool: db_pool,
    login_delay: login_delay,
  } ) )
}
