//! See https://opentelemetry.io/docs/specs/otel/protocol/exporter/
//! for environment variables that are relevant.
//!

use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::tonic_types::metadata::MetadataMap;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter, WithTonicConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::{SdkLogger, SdkLoggerProvider};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use thiserror::Error;
use tracing::Subscriber;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer};

pub fn init_logger() -> Result<(), Error> {
    let filter = EnvFilter::from_default_env();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(true)
        .with_filter(filter);

    let subscriber = Registry::default().with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn init_otlp_logger(
    app: &'static str,
    org: &str,
    stream_name: &str,
    token: &str,
) -> Result<(), Error> {
    let filter = EnvFilter::from_default_env()
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    let mut grpc_metadata = MetadataMap::with_capacity(3);

    grpc_metadata.insert("authorization", token.parse()?);
    grpc_metadata.insert("organization", org.parse()?);
    grpc_metadata.insert("stream-name", stream_name.parse()?);

    let svc_metadata = Resource::builder().with_service_name(app).build();

    let logger = logs(grpc_metadata.clone(), svc_metadata.clone())?;

    let meter = metrics(grpc_metadata.clone(), svc_metadata.clone())?;
    let tracer = tracer(app, grpc_metadata, svc_metadata)?;

    let subscriber = Registry::default()
        .with(filter)
        .with(meter)
        .with(tracer)
        .with(logger);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn logs(
    metadata: MetadataMap,
    resource: Resource,
) -> Result<OpenTelemetryTracingBridge<SdkLoggerProvider, SdkLogger>, Error> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_metadata(metadata)
        .build()
        .map_err(|source| Error::LogExporterError { source })?;

    let provider = SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    let logger = OpenTelemetryTracingBridge::new(&provider);

    Ok(logger)
}

fn metrics<S>(
    metadata: MetadataMap,
    resource: Resource,
) -> Result<MetricsLayer<S, SdkMeterProvider>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_metadata(metadata)
        .build()
        .map_err(|source| Error::MetricExporterError { source })?;

    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(resource)
        .build();

    let meter: MetricsLayer<S, SdkMeterProvider> = MetricsLayer::new(provider);

    Ok(meter)
}

fn tracer<S>(
    app: &'static str,
    metadata: MetadataMap,
    resource: Resource,
) -> Result<OpenTelemetryLayer<S, SdkTracer>, Error>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_metadata(metadata)
        .build()
        .map_err(|source| Error::TraceExporterError { source })?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer: SdkTracer = provider.tracer(app);

    let tracer: OpenTelemetryLayer<S, SdkTracer> =
        tracing_opentelemetry::layer().with_tracer(tracer);

    Ok(tracer)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid configuration, {source}")]
    ConfigurationError {
        #[from]
        source: tonic::metadata::errors::InvalidMetadataValue,
    },
    #[error("Otlp Error: Unable to build log exporter, {source}")]
    LogExporterError {
        source: opentelemetry_otlp::ExporterBuildError,
    },
    #[error("Otlp Error: Unable to build metric exporter, {source}")]
    MetricExporterError {
        source: opentelemetry_otlp::ExporterBuildError,
    },
    #[error(transparent)]
    SetGlobalDefaultError {
        #[from]
        source: tracing::dispatcher::SetGlobalDefaultError,
    },
    #[error("Otlp Error: Unable to build trace exporter, {source}")]
    TraceExporterError {
        source: opentelemetry_otlp::ExporterBuildError,
    },
}
