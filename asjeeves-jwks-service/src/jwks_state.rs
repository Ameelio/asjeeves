use std::fmt;

use crate::JsonWebKeySet;
use crate::http::HeaderValue;

pub trait JwksState: fmt::Debug {
    type Error: std::error::Error + 'static;
    fn fetch_cache_metadata(&self) -> Result<CacheMetadata, Self::Error>;
    fn fetch_client_keys(&self) -> Result<JsonWebKeySet, Self::Error>;
}

pub struct CacheMetadata {
    pub expires: HeaderValue,
    pub last_modified: HeaderValue,
}
