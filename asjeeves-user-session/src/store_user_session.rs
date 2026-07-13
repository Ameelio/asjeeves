use asjeeves_encryption::prelude::{Jws, Jwt};
use asjeeves_encryption::web_key::PrivateKey;
use axum::http::HeaderValue;
use cookie::{Cookie, SameSite};
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::Error;
use crate::generate_session_key::generate_session_key;
use crate::user_session_state::{Cache, UserSessionState};

type SessionToken = Jwt<Claims, Header>;

#[instrument(err, skip(state, user_session))]
pub async fn store_user_session<T>(
    state: &UserSessionState,
    user_session: T,
) -> Result<HeaderValue, Error>
where
    T: Serialize,
{
    let mut conn: Cache = state.cache.clone();

    let session_id: String = {
        let key: &str = &state.redis_counter_key;
        let counter: isize = conn.incr(key, 1).await?;

        counter.to_string()
    };

    let key: String = generate_session_key(state.redis_key.as_ref(), session_id.as_str());

    {
        let value: String = serde_json::to_string(&user_session)?;

        conn.set_ex(key, value, state.seconds_to_live).await?;
    }

    let value: HeaderValue = {
        let key: &PrivateKey = state.private_keys.first()?;

        let max_age = cookie::time::Duration::seconds(state.seconds_to_live as i64);

        let token: SessionToken = {
            let session_id: Box<str> = session_id.into_boxed_str();

            let claims = Claims { session_id };

            let header: Header = {
                let kid: &str = key.id();
                let kid = Box::from(kid);

                Header { kid }
            };

            SessionToken::new(claims, header)
        };

        let jws: String = {
            let jws: Jws = key
                .sign_json_web_token(token)
                .map_err(|source| Error::UnableToSignCookie { source })?;

            jws.to_string()
                .map_err(|source| Error::UnableToEncodeCookie { source })?
        };

        let cookie_value = (state.cookie_name.as_ref(), jws);

        let cookie = Cookie::build(cookie_value)
            .http_only(true)
            .max_age(max_age)
            .path("/")
            .partitioned(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .build();

        let value: String = cookie.encoded().to_string();

        value.parse()?
    };

    Ok(value)
}

#[derive(Debug, Deserialize, Serialize)]
struct Header {
    kid: Box<str>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    session_id: Box<str>,
}
