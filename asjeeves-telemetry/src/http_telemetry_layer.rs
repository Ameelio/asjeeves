use std::borrow::Cow;
use std::time::Duration;

use http::header::USER_AGENT;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode, Uri};
use http_body::Body;
use opentelemetry_semantic_conventions::attribute;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::{MakeSpan, OnRequest, OnResponse, TraceLayer};
use tracing::{Span, field, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub type HttpTelemetryLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    MakeTelemetrySpan,
    OnRequestTelemetry,
    OnResponseTelemetry,
    (),
    (),
    (),
>;

pub fn http_telemetry_layer() -> HttpTelemetryLayer {
    TraceLayer::new_for_http()
        .make_span_with(MakeTelemetrySpan {})
        .on_request(OnRequestTelemetry {})
        .on_response(OnResponseTelemetry {})
        .on_body_chunk(())
        .on_eos(())
        .on_failure(())
}

#[derive(Clone, Debug)]
pub struct MakeTelemetrySpan;

#[derive(Clone, Debug)]
pub struct OnRequestTelemetry;

#[derive(Clone, Debug)]
pub struct OnResponseTelemetry;

impl<B> MakeSpan<B> for MakeTelemetrySpan
where
    B: Body,
{
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let headers: &HeaderMap<HeaderValue> = request.headers();

        let method: &Method = request.method();
        let method = method.to_string();

        let uri: &Uri = request.uri();

        let otel_name = format!("{} {}", &method, uri.path());

        // let trace_id: TraceId = otel_ctx.span().span_context().trace_id();
        // let trace_id = format!("{:032x}", trace_id);

        info_span!(
            "HTTP Request",
            "otel.kind" = "server",
            "otel.name" = otel_name,
            "org.ameelio.http.request.headers" = ?headers,
            "org.ameelio.http.response.headers" = field::Empty,
        )
    }
}

impl<B> OnRequest<B> for OnRequestTelemetry
where
    B: Body,
{
    fn on_request(&mut self, request: &Request<B>, span: &Span) {
        let headers: &HeaderMap<HeaderValue> = request.headers();

        let method: &Method = request.method();
        let method = method.to_string();

        let uri: &Uri = request.uri();

        span.set_attribute(attribute::HTTP_REQUEST_METHOD, method);
        span.set_attribute(attribute::HTTP_ROUTE, uri.path().to_string());
        span.set_attribute(attribute::URL_FULL, uri.to_string());

        if let (Some(host), Some(port)) = snarf_server_info(headers, uri) {
            span.set_attribute(attribute::SERVER_ADDRESS, host);
            span.set_attribute(attribute::SERVER_PORT, port.to_string());
        }

        if let Some(scheme) = snarf_scheme(headers, uri) {
            span.set_attribute(attribute::URL_SCHEME, scheme);
        }

        if let Some(user_agent) = snarf_hdr(headers, USER_AGENT) {
            span.set_attribute(attribute::USER_AGENT_ORIGINAL, user_agent);
        }
    }
}

impl<B> OnResponse<B> for OnResponseTelemetry
where
    B: Body,
{
    fn on_response(self, response: &Response<B>, _latency: Duration, span: &Span) {
        let headers: &HeaderMap<HeaderValue> = response.headers();
        let status: StatusCode = response.status();

        span.set_attribute(attribute::HTTP_RESPONSE_STATUS_CODE, status.to_string());
        span.record("org.ameelio.http.response.headers", field::debug(headers));

        if status.is_server_error() {
            span.set_attribute(attribute::OTEL_STATUS_CODE, "ERROR");
        } else {
            span.set_attribute(attribute::OTEL_STATUS_CODE, "OK");
        }
    }
}

fn snarf_hdr(headers: &HeaderMap<HeaderValue>, name: HeaderName) -> Option<String> {
    let value: &HeaderValue = headers.get(name)?;
    let value: &[u8] = value.as_bytes();
    let value: Cow<'_, str> = String::from_utf8_lossy(value);
    let value = String::from(value);

    Some(value)
}

fn snarf_scheme(headers: &HeaderMap<HeaderValue>, request_uri: &Uri) -> Option<String> {
    let forwarded = HeaderName::from_static("x-forwarded-proto");
    let forwarded: Option<String> = snarf_hdr(headers, forwarded);

    match forwarded {
        Some(forwarded) => Some(forwarded),
        None => request_uri.scheme_str().map(String::from),
    }
}

fn snarf_server_info(
    headers: &HeaderMap<HeaderValue>,
    request_uri: &Uri,
) -> (Option<String>, Option<u16>) {
    let forwarded = HeaderName::from_static("x_forwarded_host");
    let forwarded: Option<String> = snarf_hdr(headers, forwarded);

    match forwarded {
        Some(forwarded) => {
            let mut forwarded = forwarded.split(':');

            let host: Option<String> = forwarded.next().map(String::from);
            let port: Option<&str> = forwarded.next();

            let port: Option<u16> = port.and_then(|s| s.parse::<u16>().ok());

            (host, port)
        }
        None => {
            let host: Option<String> = request_uri.host().map(String::from);
            (host, request_uri.port_u16())
        }
    }
}
