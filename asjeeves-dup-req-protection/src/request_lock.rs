use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use redis::aio::ConnectionLike;
use redis::{AsyncTypedCommands, SetExpiry, SetOptions};
use thiserror::Error;

/// A simple lock stored in cache. It determines if
/// a request is locked (i.e. already received) by the
/// presence of a key in cache represented by this struct.
#[derive(Clone)]
pub struct RequestLock {
    /// The path for the key, aka the prefix.
    path: Arc<str>,
    /// The duration before the key should be expired.
    ttl: Arc<Duration>,
}

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("This request is already locked.")]
    AlreadyLocked,
    #[error("Unable to set the lock, {source}")]
    UnableToSetLock {
        #[from]
        source: redis::RedisError,
    },
}

impl RequestLock {
    pub fn new(path: &str, ttl: Duration) -> Self {
        let path: Arc<str> = path.into();
        let ttl = Arc::new(ttl);

        Self { path, ttl }
    }

    /// Verifies if this is a duplicate request by attempting to create a lock.
    pub async fn try_lock<C>(&self, conn: &mut C, nonce: &str) -> Result<(), Error>
    where
        C: ConnectionLike + AsyncTypedCommands,
    {
        let key = format!("{}/{}", self.path, nonce);

        let ttl: u64 = self.ttl.as_secs();
        let ttl = SetExpiry::EX(ttl);

        let options = SetOptions::default()
            .conditional_set(redis::ExistenceCheck::NX)
            .with_expiration(ttl);

        // retry ONCE in case its just a dropped connection.
        let res: Option<String> = match conn.set_options(&key, true, options.clone()).await {
            Ok(res) => res,
            _ => conn.set_options(key, true, options).await?,
        };

        res.ok_or(Error::AlreadyLocked)?;

        Ok(())
    }
}

impl fmt::Debug for RequestLock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RequestLock")
            .field("path", &self.path)
            .field("ttl", &self.ttl)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use redis_test::{MockCmd, MockRedisConnection};

    use super::*;

    #[tokio::test]
    async fn it_should_lock() {
        let options = SetOptions::default()
            .conditional_set(redis::ExistenceCheck::NX)
            .with_expiration(SetExpiry::EX(1));

        let mut cmd = redis::cmd("SET");

        cmd.arg("foo/foo");
        cmd.arg(b"1");
        cmd.arg(options);

        let mut conn = MockRedisConnection::new(vec![
            MockCmd::new(cmd.clone(), Ok(redis::Value::Okay)),
            MockCmd::new(cmd, Ok(redis::Value::Nil)),
        ]);

        let ttl = Duration::from_secs(1);

        let req_lock = RequestLock::new("foo", ttl);

        assert_eq!(Ok(()), req_lock.try_lock(&mut conn, "foo").await);
        assert_eq!(
            Err(Error::AlreadyLocked),
            req_lock.try_lock(&mut conn, "foo").await
        );
    }
}
