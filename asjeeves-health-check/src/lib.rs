//! Basic [tower::Service] that returns a 200 OK response.
//! indicating this service is Healthy.

use std::convert::Infallible;
use std::future::Ready;
use std::task::Poll;

use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use tower::Service;

const OK: &'static str = "OK";

#[derive(Clone)]
pub struct HealthCheck {}

impl HealthCheck {
    fn respond_ok() -> Result<Response<Full<Bytes>>, Infallible> {
        let body = Bytes::from(OK);
        let body = Full::new(body);

        let response = Response::builder()
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .status(StatusCode::OK)
            .body(body)
            .expect("This should only fail if the status or headers are invalid, which is not the case here");

        Ok(response)
    }
}

// This implements the tower Service interface so we can
// add this to the router.
impl<T> Service<Request<T>> for HealthCheck {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    // This is the entry point which handles the request.
    fn call(&mut self, _req: Request<T>) -> Self::Future {
        std::future::ready(Self::respond_ok())
    }

    // Rust's asynchronous code requires a poll_ready method.
    // Because we do not do any blocking I/O or syscalls we
    // can just have it always ready.
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_responds_with_ok() {
        let resp = HealthCheck::respond_ok().unwrap();

        assert_eq!(StatusCode::OK, resp.status());
    }
}
