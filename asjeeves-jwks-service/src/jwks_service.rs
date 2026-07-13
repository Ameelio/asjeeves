use std::convert::Infallible;
use std::fmt;
use std::future::{Ready, ready};
use std::sync::Arc;
use std::task::{Context, Poll};

use bytes::Bytes;
use http::{Request, Response, StatusCode, header};
use http_body_util::Full;
use json_web_key::jwk::JsonWebKeySet;
use tower::Service;
use tracing::instrument;

use crate::{CacheMetadata, JwksState};

#[derive(Clone, Debug)]
pub struct JwksService {
    pub state: Arc<dyn JwksState>,
}

impl JwksService {
    fn error_response() -> Result<Response<Full<Bytes>>, Infallible> {
        let body = Bytes::default();
        let body = Full::new(body);

        let resp = Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap();

        Ok(resp)
    }

    #[instrument(err)]
    fn get_jwks_handler(&self) -> Result<Response<Full<Bytes>>, Infallible> {
        let Ok(metadata): Result<CacheMetadata, _> = self.state.fetch_cache_metadata() else {
            return Self::error_response();
        };

        let body: Full<Bytes> = {
            let Ok(jwks): Result<JsonWebKeySet, _> = self.state.fetch_client_keys() else {
                return Self::error_response();
            };

            let Ok(bytes) = serde_json::to_vec(&jwks) else {
                return Self::error_response();
            };

            let bytes = Bytes::from(bytes);

            Full::new(bytes)
        };

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::EXPIRES, metadata.expires)
            .header(header::LAST_MODIFIED, metadata.last_modified)
            .body(body)
            .unwrap();

        Ok(response)
    }
}

impl<B> Service<Request<B>> for JwksService
where
    B: fmt::Debug,
{
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    #[instrument]
    fn call(&mut self, _: Request<B>) -> Self::Future {
        ready(self.get_jwks_handler())
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashSet, convert::Infallible};

    use http::HeaderValue;
    use json_web_key::{jwk::JsonWebKey, rsa::RsaWebKey};

    use super::*;

    #[derive(Clone, Debug)]
    pub struct StateWithJwks;

    impl JwksState for StateWithJwks {
        fn fetch_cache_metadata(&self) -> Result<CacheMetadata, Box<dyn std::error::Error>> {
            let data = CacheMetadata {
                expires: HeaderValue::from_static("2026-01-01T13:00.00Z"),
                last_modified: HeaderValue::from_static("2026-01-01T13:00.00Z"),
            };

            Ok(data)
        }

        fn fetch_client_keys(&self) -> Result<JsonWebKeySet, Box<dyn std::error::Error>> {
            let jwk = RsaWebKey::default();
            let jwk = JsonWebKey::RS256(jwk);
            let mut keys: HashSet<JsonWebKey> = HashSet::with_capacity(1);
            keys.insert(jwk);

            let set = JsonWebKeySet { keys };

            Ok(set)
        }
    }

    #[test]
    fn it_provides_jwks() {
        let state = StateWithJwks {};
        let serv = JwksService {
            state: Arc::new(state),
        };

        let resp = serv.get_jwks_handler().unwrap();

        assert_eq!(StatusCode::OK, resp.status())
    }
}
