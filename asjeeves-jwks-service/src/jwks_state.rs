use std::fmt;

use crate::JsonWebKeySet;
use crate::http::HeaderValue;

pub trait JwksState: fmt::Debug + Send + Sync {
    fn fetch_cache_metadata(&self) -> Result<CacheMetadata, Box<dyn std::error::Error>>;
    fn fetch_client_keys(&self) -> Result<JsonWebKeySet, Box<dyn std::error::Error>>;
}

pub struct CacheMetadata {
    pub expires: HeaderValue,
    pub last_modified: HeaderValue,
}
