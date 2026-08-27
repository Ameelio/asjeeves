use std::{convert::Infallible, fmt};

use axum::{
    http::{StatusCode, header::InvalidHeaderValue},
    response::IntoResponse,
};
use thiserror::Error;

/// User Session Error
#[derive(Debug, Error)]
pub enum Error {
    #[error("this should never happen, infallible does not fail")]
    InfallibleFailed {
        #[from]
        source: Infallible,
    },
    #[error("invalid user session cookie")]
    InvalidCookieSignature,
    #[error("missing session cookie")]
    MissingSessionCookie,
    #[error("no keys to sign session cookie")]
    MissingSigningKey,
    #[error("UserSessionLayer is missing, please add it to your Router")]
    MissingUserSessionLayer,
    #[error("session not found")]
    SessionNotFound,
    #[error("an unexpected cache error, {source}")]
    UnexpectedCacheError {
        #[from]
        source: redis::RedisError,
    },
    #[error("unable to encode cookie, {source}")]
    UnableToEncodeCookie {
        source: asjeeves_encryption::jwt::error::Error,
    },
    #[error("unable to set session cookie, {source}")]
    UnableToSetCookie {
        #[from]
        source: InvalidHeaderValue,
    },
    #[error("unable to sign cookie, {source}")]
    UnableToSignCookie { source: asjeeves_encryption::Error },
    #[error("unable to parse session data, {source}")]
    UnableToParseSession { source: serde_json::Error },
    #[error("unable to parse signature, {source}")]
    UnableToParseSignature {
        source: asjeeves_encryption::jwt::error::Error,
    },
    #[error("unable to parse token, {source}")]
    UnableToParseToken {
        source: asjeeves_encryption::jwt::error::Error,
    },
    #[error("an unexpected error with (de)serialization, {source}")]
    UnexpectedSerdeJsonError {
        #[from]
        source: serde_json::Error,
    },
}

#[derive(Clone, Debug)]
pub enum ErrorResponse {
    InternalError(Box<str>),
    SessionNotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let message: Box<str> = self.to_string().into();

        match self {
            Self::SessionNotFound => {
                let mut response = StatusCode::UNAUTHORIZED.into_response();

                response
                    .extensions_mut()
                    .insert(ErrorResponse::SessionNotFound);

                response
            }
            _ => {
                let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

                response
                    .extensions_mut()
                    .insert(ErrorResponse::InternalError(message));

                response
            }
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionNotFound => write!(f, "unauthorized"),
            Self::InternalError(message) => write!(f, "{}", message),
        }
    }
}
