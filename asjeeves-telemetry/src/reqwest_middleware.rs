use std::borrow::Cow;

use http::Extensions;
use reqwest::{Request, Response};
use tracing::Span;

use reqwest_tracing::{
    ReqwestOtelSpanBackend, TracingMiddleware, default_on_request_end, reqwest_otel_span,
};

pub type HttpTraceMiddleware = TracingMiddleware<HttpTrace>;

/// Reqwest middleware that traces http requests and includes the body in the payload.
/// ## Example
///   use crate::http_trace::HttpTrace;
///   use reqwest_tracing::TracingMiddleware;
///
///   let http_client = reqwest::Client::builder().build()?;
///
///   let http_client = reqwest_middleware::ClientBuilder::new(http_client)
///     .with(TracingMiddleware::<HttpTrace>::new())
///     .build();
pub struct HttpTrace;

impl ReqwestOtelSpanBackend for HttpTrace {
    fn on_request_start(req: &Request, _extension: &mut Extensions) -> Span {
        let name = format!("REQUEST {} {}", req.method(), req.url());

        let body: Cow<'_, str> = req
            .body()
            .and_then(|x| x.as_bytes())
            .map(|x| String::from_utf8_lossy(x))
            .unwrap_or_default();

        let body: &str = &body;

        reqwest_otel_span!(name = name, req, body)
    }

    fn on_request_end(
        span: &Span,
        outcome: &reqwest_middleware::Result<Response>,
        _extension: &mut Extensions,
    ) {
        default_on_request_end(span, outcome);
    }
}
