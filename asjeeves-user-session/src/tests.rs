use std::sync::Arc;

use asjeeves_encryption::seed::Seed;
use asjeeves_encryption::web_key::PrivateKey;
use axum::body::Body;
use axum::extract::FromRef;
use axum::http::{Request, Response};
use axum::middleware::Next;
use redis::aio::ConnectionManager;

use crate::error::ErrorResponse;
use crate::user_session_keys::UserSessionKeys;

#[derive(Clone, FromRef)]
pub struct TestState {
    pub cache: ConnectionManager,
    pub keys: UserSessionKeys,
}

pub async fn test_state() -> TestState {
    let cache = {
        let redis_url = std::env::var("TEST_REDIS_URL").expect("env TEST_REDIS_URL is missing");

        let client = redis::Client::open(redis_url).unwrap();

        ConnectionManager::new(client).await.unwrap()
    };

    let keys: UserSessionKeys = {
        let seed = Seed::from(1);
        let mut rng = seed.rng();

        let key = PrivateKey::generate(&mut rng).unwrap();

        let keys = vec![key];

        let keys: Arc<[PrivateKey]> = keys.into();

        UserSessionKeys(keys)
    };

    TestState { cache, keys }
}

pub async fn error_response_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(req).await;

    let err: Option<ErrorResponse> = response.extensions_mut().remove();

    let Some(err) = err else {
        return response;
    };

    let message: Body = err.to_string().into();

    response.map(|_| message)
}
