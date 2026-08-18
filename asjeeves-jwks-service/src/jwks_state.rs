use std::fmt;
use std::future::Future;
use std::pin::Pin;

use crate::JsonWebKeySet;
use crate::http::HeaderValue;

type BoxFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error>>> + Send + 'a>>;

pub trait JwksState: fmt::Debug + Send + Sync {
    fn fetch_cache_metadata(&self) -> BoxFuture<'_, CacheMetadata>;
    fn fetch_client_keys(&self) -> BoxFuture<'_, JsonWebKeySet>;
}

pub struct CacheMetadata {
    pub expires: HeaderValue,
    pub last_modified: HeaderValue,
}
