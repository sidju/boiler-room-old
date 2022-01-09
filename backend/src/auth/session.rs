use crate::Error;
use crate::State;

// The struct given to each handler
// It should contain everything needed to know
// the user and its permissions
#[derive(Debug)]
pub struct Permissions {
    // For use by html rendering handlers to print in top-bar
    pub username: String,
    // To identify if the current user owns a resource
    pub userid: i32,
    // To identify if the current user has admin perms
    pub admin: bool,
}

// An async task that clears out outdated sessions every hour
// Runs indefinitely
pub async fn prune_sessions(state: &'static State) {
    loop {
        // Run the cleanup query
        sqlx::query!("DELETE FROM sessions WHERE until < NOW()")
            .execute(&state.db_pool)
            .await
            .expect("Failed to prune sessions!");

        // Delay for one hour before doing again
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

// Check an optionally provided session key
pub async fn session(
    state: &'static State,
    key: Option<String>,
) -> Result<Option<Permissions>, Error> {
    if let Some(key) = key {
        let sess = sqlx::query_as!(
            Permissions,
            "
SELECT username, userid, admin
FROM sessions
JOIN users ON sessions.userid = users.id
WHERE sessions.key = $1 AND sessions.until > NOW()
      ",
            key,
        )
        .fetch_optional(&state.db_pool)
        .await?;
        Ok(sess)
    } else {
        Ok(None)
    }
}
// Check a required session key and error if not valid
pub async fn require_session(
    state: &'static State,
    key: Option<String>,
) -> Result<Permissions, Error> {
    let sess = session(state, key).await?;

    match sess {
        Some(s) => Ok(s),
        None => Err(Error::unauthorized()),
    }
}
// Check the required session key and error if invalid or admin
pub async fn require_admin(
    state: &'static State,
    key: Option<String>,
) -> Result<Permissions, Error> {
    // First we require a session
    let data = require_session(state, key).await?;
    // Then, if not admin, we error
    if data.admin {
        Ok(data)
    } else {
        Err(Error::forbidden())
    }
}
