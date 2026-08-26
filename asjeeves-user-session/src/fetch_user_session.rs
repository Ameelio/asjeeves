use std::borrow::Cow;

use asjeeves_encryption::prelude::*;
use cookie::Cookie;
use redis::AsyncTypedCommands;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::error::Error;
use crate::generate_session_key::generate_session_key;
use crate::user_session_state::{Cache, UserSessionState};

type SessionToken = Jwt<Claims, Header>;

/// Fetches user session data from redis.
#[instrument(err)]
pub async fn fetch_user_session<T>(cookie: Cookie<'_>, state: &UserSessionState) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let session_id: Box<str> = {
        let jws: Jws = {
            let value: &str = cookie.value();

            tracing::info!("signature {}", value);

            value
                .try_into()
                .map_err(|source| Error::UnableToParseSignature { source })?
        };

        let token: SessionToken = {
            let token: Cow<str> = jws.encoded_token();

            token
                .as_ref()
                .try_into()
                .map_err(|source| Error::UnableToParseToken { source })?
        };

        let kid: Box<str> = token.header().kid.clone();

        let key: &PrivateKey = state.private_keys.find(&kid)?;

        key.verify_json_web_signature(&jws)
            .map_err(|_| Error::InvalidCookieSignature)?;

        token.claims().session_id.clone()
    };

    let user_session: T = {
        let mut conn: Cache = state.cache.clone();

        let key: String = generate_session_key(&state.redis_key, &session_id);

        let user_session: String = conn.get(&key).await?.ok_or(Error::SessionNotFound)?;

        serde_json::from_str(user_session.as_str())
            .map_err(|source| Error::UnableToParseSession { source })?
    };

    Ok(user_session)
}

#[derive(Deserialize, Debug, Serialize)]
struct Claims {
    session_id: Box<str>,
}

#[derive(Deserialize, Debug, Serialize)]
struct Header {
    kid: Box<str>,
}
