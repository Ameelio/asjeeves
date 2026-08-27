use std::fmt;
use std::sync::Arc;

use asjeeves_encryption::prelude::*;

use crate::error::Error;

/// A thread safe list of [PrivateKey].
// We use a newtype here so we can extract it from a state using FromRef.
#[derive(Clone)]
pub struct UserSessionKeys(pub Arc<[PrivateKey]>);

impl UserSessionKeys {
    pub fn find(&self, id: &str) -> Result<&PrivateKey, Error> {
        let keys: &[PrivateKey] = &self.0;

        let key: Option<&PrivateKey> = keys.iter().find(|x| x.id() == id);

        key.ok_or(Error::MissingSigningKey)
    }

    pub fn first(&self) -> Result<&PrivateKey, Error> {
        let keys: &[PrivateKey] = &self.0;

        let key: Option<&PrivateKey> = keys.first();

        key.ok_or(Error::MissingSigningKey)
    }
}

impl fmt::Debug for UserSessionKeys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSessionKeys").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod test {
    use crate::tests::test_state;

    #[tokio::test]
    async fn it_should_obfuscate_debug_info() {
        let ts = test_state().await;

        let usk = ts.keys;

        let dbg = format!("{:?}", usk);

        assert_eq!("UserSessionKeys { .. }", dbg);
    }
}
