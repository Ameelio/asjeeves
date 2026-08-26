use std::sync::Arc;

use chrono::TimeDelta;
use tower::Layer;

use crate::user_session_keys::UserSessionKeys;
use crate::user_session_options::UserSessionOptions;
use crate::user_session_service::UserSessionService;
use crate::user_session_state::{Cache, UserSessionState};

/// A middleware service that loads or stores a [UserSession].
#[derive(Clone)]
pub struct UserSessionLayer {
    pub cache: Cache,
    /// Optional overrides to the default configuration.
    pub options: UserSessionOptions,
    /// Private Keys for signing and verifying
    pub private_keys: UserSessionKeys,
    /// Time to live for the user session date.
    pub ttl: TimeDelta,
}

impl<InnerService> Layer<InnerService> for UserSessionLayer {
    type Service = UserSessionService<InnerService>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        let cache: Cache = self.cache.clone();
        let cookie_name: Arc<str> = Arc::clone(&self.options.cookie_name);
        let redis_counter_key: Arc<str> = Arc::clone(&self.options.redis_counter_key);
        let redis_key: Arc<str> = Arc::clone(&self.options.redis_key);
        let private_keys: UserSessionKeys = self.private_keys.clone();
        let seconds_to_live: u64 = self.ttl.num_seconds().unsigned_abs();

        let state = UserSessionState {
            cache,
            cookie_name,
            redis_counter_key,
            redis_key,
            private_keys,
            seconds_to_live,
        };

        UserSessionService { inner, state }
    }
}
