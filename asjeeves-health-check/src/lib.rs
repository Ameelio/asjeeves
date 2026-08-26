//! Basic [tower::Service][tower] that returns a 200 OK response.
//! indicating this service is Healthy.
//!
//! ## Example
//! Using axum.
//! ```
//! use asjeeves_health_check::HealthCheck;
//! use axum::Router;
//!
//! #[derive(Clone)]
//! struct WebState {}
//!
//! let app : Router<WebState> = Router::new()
//!     .route_service("/health", HealthCheck {})
//!     .with_state(WebState {});
//!
//! ```

use std::convert::Infallible;
use std::future::Ready;
use std::task::Poll;

use bytes::Bytes;
use http::header::CONTENT_TYPE;
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use tower::Service;

const OK: &str = "OK";

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
    use std::pin::Pin;
    use std::task::{Context, Waker};

    use super::*;

    #[test]
    fn it_responds_with_ok() {
        let mut service = HealthCheck {};

        let request = Request::builder().body(()).unwrap();

        let mut cx = Context::from_waker(Waker::noop());

        let poll = <HealthCheck as Service<Request<()>>>::poll_ready(&mut service, &mut cx);
        assert!(poll.is_ready());

        let mut future = service.call(request);

        let poll = Future::poll(Pin::new(&mut future), &mut cx);

        match poll {
            Poll::Ready(Ok(response)) => assert_eq!(StatusCode::OK, response.status()),
            _ => panic!("Expected service to respond with OK"),
        }
    }
}
