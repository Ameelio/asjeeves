use std::fmt;
use std::sync::Arc;

use crate::user_session_keys::UserSessionKeys;

pub type Cache = redis::aio::ConnectionManager;

#[derive(Clone)]
pub struct UserSessionState {
    // the redis ConnectionManager can be cloned cheaply
    // and safely.
    pub cache: Cache,
    /// Name of the user session cookie defaults to `session-id`
    pub cookie_name: Arc<str>,
    /// Location in redis for the session counter, used to increment a value and garuntee
    /// uniqueness.
    pub redis_counter_key: Arc<str>,
    /// Location in redis (key) the user sessions are stored, defaults to `/sessions`
    pub redis_key: Arc<str>,
    /// Private Keys to use for signing and verifying
    pub private_keys: UserSessionKeys,
    /// Time to live for the user session data.
    pub seconds_to_live: u64,
}

// we dont show debug info for cache because that
// has the conn info which is sensitive.
impl fmt::Debug for UserSessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionStore")
            .field("cookie_name", &self.cookie_name)
            .field("redis_counter_key", &self.redis_counter_key)
            .field("redis_key", &self.redis_key)
            .field("seconds_to_live", &self.seconds_to_live)
            .finish_non_exhaustive()
    }
}
