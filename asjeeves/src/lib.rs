//! Ameelio services Jeeves
//!
//! ## Features
//! - **axum** CSRF middleware (includes tower feature)
//! - **dup_req_protection** Duplicate Request Protection
//! - **encryption** Utilities for encryption and signature validation.
//! - **logger** An OTLP friendly logger.
//! - **reqwest** HTTP Request tracing for the `reqwest` crate.
//! - **tower** Health Check and JWKS web services that use `tower`.
//! - **zipstream** A zip compressor that works over a stream.
//!
//! ## Documentation
//! - [`asjeeves_csrf`] - CSRF protection middleware for <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate feature `axum` only.</mark>
//! - [`asjeeves_dup_req_protection`] - Duplicate request protection <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate feature `dup_req_protection` only.</mark>
//! - [`asjeeves_encryption`] - Encryption and signature validation utilities <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate feature `encryption` only.</mark>
//! - [`asjeeves_health_check`] - Health check service for Tower <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate features `axum` and `tower`.</mark>
//! - [`asjeeves_jwks_service`] - JWKS (JSON Web Key Set) service for Tower <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate features `axum` and `tower`.</mark>
//! - [`asjeeves_telemetry`] - OpenTelemetry integration.
//! - [`asjeeves_user_session`] - User session management for Axum <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate feature `axum` only.</mark>
//! - [`asjeeves_zipstream`] - Streaming zip compression <mark style="background-color: #fff3cd; color: #856404; padding: 2px 6px; border-radius: 3px; font-size: 85%;">Available on crate feature `zipstream` only.</mark>

#[cfg(feature = "axum")]
pub mod csrf {
    pub use asjeeves_csrf::*;
}

#[cfg(feature = "dup_req_protection")]
pub mod dup_req_protection {
    pub use asjeeves_dup_req_protection::*;
}

#[cfg(feature = "encryption")]
pub mod encryption {
    pub use asjeeves_encryption::*;
}

#[cfg(feature = "tower")]
pub mod health_check {
    pub use asjeeves_health_check::*;
}

#[cfg(feature = "tower")]
pub mod jwks_service {
    pub use asjeeves_jwks_service::*;
}

#[cfg(feature = "telemetry")]
pub mod telemetry {
    pub use asjeeves_telemetry::*;
}

#[cfg(feature = "axum")]
pub mod user_session {
    pub use asjeeves_user_session::*;
}

#[cfg(feature = "zipstream")]
pub mod zipstream {
    pub use asjeeves_zipstream::*;
}

pub mod prelude {
    #[cfg(feature = "axum")]
    pub use super::axum::*;

    #[cfg(feature = "dup_req_protection")]
    pub use asjeeves_dup_req_protection::RequestLock;

    #[cfg(feature = "encryption")]
    pub use asjeeves_encryption::prelude::*;

    #[cfg(feature = "tower")]
    pub use super::tower::*;
}

#[cfg(feature = "axum")]
mod axum {
    pub use asjeeves_csrf::FormAuthenticityToken;
    pub use asjeeves_user_session::prelude::*;
}

#[cfg(feature = "tower")]
mod tower {
    pub use asjeeves_health_check::HealthCheck;
    pub use asjeeves_jwks_service::JwksService;
}
