use std::fmt;
use std::sync::Arc;

use crate::user_session_keys::UserSessionKeys;

pub type Cache = redis::aio::ConnectionManager;

/// Stores connection pools, keys, and configuration for [crate::user_session_service::UserSessionService].
#[derive(Clone)]
pub struct UserSessionState {
    /// the redis ConnectionManager can be cloned cheaply
    /// and safely.
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::test_state;

    #[tokio::test]
    async fn it_should_obfuscate_when_debugged() {
        let ts = test_state().await;

        let state = UserSessionState {
            cache: ts.cache,
            cookie_name: "test".into(),
            private_keys: ts.keys,
            redis_counter_key: "test".into(),
            redis_key: "test".into(),
            seconds_to_live: 0,
        };

        let dbg = format!("{:?}", state);

        assert_eq!(
            r#"SessionStore { cookie_name: "test", redis_counter_key: "test", redis_key: "test", seconds_to_live: 0, .. }"#,
            dbg
        );
    }
}
