//! Ameelio services Jeeves
//!
//! ## Features
//!     - **axum** CSRF middleware (includes tower feature)
//!     - **dup_req_protection** Duplicate Request Protection
//!     - **encryption** Utilities for encryption and signature validation.
//!     - **logger** An OTLP friendly logger.
//!     - **reqwest** HTTP Request tracing for the `reqwest` crate.
//!     - **tower** Health Check and JWKS web services that use `tower`.
//!     - **zipstream** A zip compressor that works over a stream.
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
