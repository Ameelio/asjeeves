use std::convert::Infallible;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
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
    async fn get_jwks_handler(
        state: Arc<dyn JwksState>,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        let Ok(metadata): Result<CacheMetadata, _> = state.fetch_cache_metadata().await else {
            return Self::error_response();
        };

        let body: Full<Bytes> = {
            let Ok(jwks): Result<JsonWebKeySet, _> = state.fetch_client_keys().await else {
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
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[instrument(skip(self))]
    fn call(&mut self, _: Request<B>) -> Self::Future {
        let state = self.state.clone();

        Box::pin(async move { Self::get_jwks_handler(state).await })
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::pin::Pin;
    use std::task::{Context, Waker};

    use http::HeaderValue;
    use json_web_key::{jwk::JsonWebKey, rsa::RsaWebKey};

    use super::*;

    #[derive(Clone, Debug)]
    pub struct StateWithJwks;

    impl JwksState for StateWithJwks {
        fn fetch_cache_metadata(
            &self,
        ) -> Pin<
            Box<dyn Future<Output = Result<CacheMetadata, Box<dyn std::error::Error>>> + Send + '_>,
        > {
            Box::pin(async {
                let data = CacheMetadata {
                    expires: HeaderValue::from_static("2026-01-01T13:00.00Z"),
                    last_modified: HeaderValue::from_static("2026-01-01T13:00.00Z"),
                };

                Ok(data)
            })
        }

        fn fetch_client_keys(
            &self,
        ) -> Pin<
            Box<dyn Future<Output = Result<JsonWebKeySet, Box<dyn std::error::Error>>> + Send + '_>,
        > {
            Box::pin(async {
                let jwk = RsaWebKey::default();
                let jwk = JsonWebKey::RS256(jwk);
                let mut keys: HashSet<JsonWebKey> = HashSet::with_capacity(1);
                keys.insert(jwk);

                let set = JsonWebKeySet { keys };

                Ok(set)
            })
        }
    }

    #[test]
    fn it_provides_jwks() {
        let mut cx = Context::from_waker(Waker::noop());
        let request = Request::builder().body(()).unwrap();
        let state = StateWithJwks {};

        let mut service = JwksService {
            state: Arc::new(state),
        };

        let poll = <JwksService as Service<Request<()>>>::poll_ready(&mut service, &mut cx);

        assert!(poll.is_ready());

        let mut future = service.call(request);

        let poll = Future::poll(Pin::new(&mut future), &mut cx);

        match poll {
            Poll::Ready(Ok(response)) => assert_eq!(StatusCode::OK, response.status()),
            _ => panic!("Expected service to respond with OK"),
        }
    }
}
