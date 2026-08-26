use std::fmt;

use axum::extract::FromRequestParts;
use axum::response::{IntoResponse, IntoResponseParts, ResponseParts};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::Error;
use crate::fetch_user_session::fetch_user_session;
use crate::user_session_state::UserSessionState;

/// An axum extractor for user session data.
#[derive(Clone)]
pub struct UserSession<T: Clone>(pub T);

impl<T> fmt::Debug for UserSession<T>
where
    T: Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSession").finish_non_exhaustive()
    }
}

impl<S, T> FromRequestParts<S> for UserSession<T>
where
    S: Send + Sync,
    T: Clone + for<'de> Deserialize<'de> + Send + Sync,
{
    type Rejection = Error;

    #[instrument(err, skip(state))]
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state).await?;

        let Some(uss): Option<&UserSessionState> = parts.extensions.get() else {
            return Err(Error::MissingUserSessionLayer);
        };

        let Some(cookie): Option<Cookie> = jar.get(&uss.cookie_name).cloned() else {
            return Err(Error::MissingSessionCookie);
        };

        let user_session: T = fetch_user_session(cookie, uss).await?;

        Ok(UserSession(user_session))
    }
}

impl<T> IntoResponseParts for UserSession<T>
where
    T: Clone + Serialize,
{
    type Error = Error;

    #[instrument(err)]
    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let user_session: UserSession<serde_json::Value> = {
            let inner: serde_json::Value = serde_json::to_value(self.0)?;

            UserSession(inner)
        };

        res.extensions_mut().insert(user_session);

        Ok(res)
    }
}

impl<T> IntoResponse for UserSession<T>
where
    T: Clone + Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (self, ()).into_response()
    }
}
