use std::sync::Arc;

/// Optional configuration for user sessions.
///
/// These defaults are set to what should be fine 90% of the time, but in edge cases
/// can be changed in situations like, multiple user sessions.
/// ## Example
///     use std::sync::Arc;
///
///     use asjeeves_user_session::UserSessionOptions;
///
///     // Set each option
///     let cookie_name: Arc<str> = "user-session-id".into();
///     let redis_counter_key: Arc<str> = "/user_sessions/counter".into();
///     let redis_key: Arc<str> = "/user_sessions/store".into();
///     let options = UserSessionOptions {
///         cookie_name,
///         redis_counter_key,
///         redis_key
///     };
///
///     // Or just change one option.
///     let options = UserSessionOptions {
///         cookie_name: "user-session-id".into(),
///         ..UserSessionOptions::default()
///     };
#[derive(Clone, Debug)]
pub struct UserSessionOptions {
    /// Name of the user session cookie defaults to `session-id`
    pub cookie_name: Arc<str>,
    /// Location in redis for the session counter, used to increment a value and garuntee
    /// uniqueness.
    pub redis_counter_key: Arc<str>,
    /// Location in redis (key) the user sessions are stored, defaults to `/sessions`
    pub redis_key: Arc<str>,
}

impl Default for UserSessionOptions {
    fn default() -> Self {
        let cookie_name: Arc<str> = "user-session-id".into();
        let redis_counter_key: Arc<str> = "/user_sessions/counter".into();
        let redis_key: Arc<str> = "/user_sessions/store".into();

        Self {
            cookie_name,
            redis_counter_key,
            redis_key,
        }
    }
}
