#[cfg(feature = "axum")]
pub mod axum_metrics;
pub mod database_metrics;
#[cfg(feature = "tower")]
pub mod http_telemetry_layer;
#[cfg(feature = "logger")]
pub mod logger;
#[cfg(feature = "reqwest")]
pub mod reqwest_middleware;
#[cfg(feature = "error")]
pub mod traceable_error;

#[cfg(feature = "axum")]
pub use axum_metrics::metrics_middleware;
#[cfg(feature = "bb8")]
pub use database_metrics::track_database_metrics;
pub use database_metrics::{db_metrics_setup, time_async_query};
#[cfg(feature = "tower")]
pub use http_telemetry_layer::{HttpTelemetryLayer, http_telemetry_layer};
#[cfg(feature = "logger")]
pub use logger::{init_logger, init_otlp_logger};
#[cfg(feature = "error")]
pub use traceable_error::{TraceableError, trace_error};
