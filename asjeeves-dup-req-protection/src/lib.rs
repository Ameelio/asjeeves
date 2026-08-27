//! Duplicate Request Protection.
//!
//! Provides a cache based locking system, provided a nonce unique to the request the
//! lock can protect a site from getting spam requests, or protect users in the case of
//! initating multiple requests over a short span of time.
mod request_lock;

pub use request_lock::{Error, RequestLock};
