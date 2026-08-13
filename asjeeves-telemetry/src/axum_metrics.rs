use std::time::Instant;

use axum::extract::{MatchedPath, Request};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use http::StatusCode;
use metrics::histogram;

/// Axum middleware that tracks request duration.
pub async fn metrics_middleware(
    method: Method,
    route: MatchedPath,
    req: Request,
    next: Next,
) -> Response {
    let method: String = method.to_string();
    let scheme: String = req.uri().scheme_str().unwrap_or("http").to_string();
    let route: String = route.as_str().to_string();

    let timer = Instant::now();

    let response: Response = next.run(req).await;

    let latency: f64 = timer.elapsed().as_secs_f64();
    let status: StatusCode = response.status();

    let req_duration = histogram!(
        description: "HTTP Request Duration",
        unit: metrics::Unit::Seconds,
        "http.server.request.duration",
        "http.request.method" => method,
        "url.scheme" => scheme,
        "http.route" => route,
        "http.response.status_code" => status.to_string()
    );

    req_duration.record(latency);

    response
}
