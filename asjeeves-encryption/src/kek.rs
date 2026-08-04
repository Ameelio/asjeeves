use core::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64ct::{Base64, Encoding};
use rand_core::{CryptoRng, RngCore};
use serde::Deserialize;
use tracing::instrument;
use zeroize::Zeroizing;

use crate::error::Error;
use crate::web_key::WebKey;

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct KeyEncryptionKey(Arc<aes_gcm::Key<Aes256Gcm>>);

pub struct EncryptedKey {
    pub key: Box<[u8]>,
    pub key_id: Box<str>,
    pub nonce: Box<[u8]>,
}

impl KeyEncryptionKey {
    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: CryptoRng + RngCore,
    {
        let key = Aes256Gcm::generate_key(rng);

        let key = Arc::new(key);

        Self(key)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    #[instrument]
    pub fn encrypt_key<R>(&self, key: &WebKey, rng: &mut R) -> Result<EncryptedKey, Error>
    where
        R: CryptoRng + RngCore + Debug,
    {
        let cipher = Aes256Gcm::new(&self.0);

        let key_id: Box<str> = {
            let key_id: &str = key.id();

            key_id.into()
        };

        let nonce = Aes256Gcm::generate_nonce(rng);
        let key_vec: Zeroizing<Vec<u8>> = key.to_bytes()?;

        let key: Box<[u8]> = cipher
            .encrypt(&nonce, key_vec.as_slice())
            .map_err(|source| Error::AesEncryptionError { source })?
            .as_slice()
            .into();

        Ok(EncryptedKey {
            key_id,
            key,
            nonce: nonce.as_slice().into(),
        })
    }

    #[instrument(skip(enc_key, nonce))]
    pub fn decrypt_key(&self, enc_key: &[u8], key_id: &str, nonce: &[u8]) -> Result<WebKey, Error> {
        let cipher = Aes256Gcm::new(&self.0);
        let nonce = Nonce::from_slice(nonce);

        // Use Zeroizing to ensure the bytes are zeroed out immediately.
        let dec_key: Zeroizing<Vec<u8>> = cipher
            .decrypt(nonce, enc_key)
            .map_err(|source| Error::AesDecryptionError { source })?
            .into();

        let key_id: Arc<str> = key_id.into();

        let web_key = WebKey::from_bytes(dec_key, key_id)?;

        Ok(web_key)
    }
}

impl fmt::Debug for KeyEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyEncryptionKey").finish_non_exhaustive()
    }
}

impl fmt::Display for KeyEncryptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b64str: String = Base64::encode_string(&self.0);

        write!(f, "{}", b64str)
    }
}

impl TryFrom<String> for KeyEncryptionKey {
    type Error = base64ct::Error;

    #[instrument]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // 256-bit AES requires the key to be 32 bytes in length.
        let mut key = [0u8; 32];

        // This will throw an error if it does not decode exactly 32 bytes.
        Base64::decode(value, &mut key)?;

        let key: aes_gcm::Key<Aes256Gcm> = key.into();
        let key = Arc::new(key);

        Ok(Self(key))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::seed::{Rng, Seed};

    #[test]
    fn it_should_encrypt_and_decrypt_keys() {
        let seed = Seed::from(1);
        let mut rng: Rng = seed.rng();

        let web_key = WebKey::generate(&mut rng).unwrap();
        let kek = KeyEncryptionKey::generate(&mut rng);

        let payload = kek.encrypt_key(&web_key, &mut rng).unwrap();
        let nonce = payload.nonce;
        let enc_key = payload.key;

        let dec_key = kek.decrypt_key(&enc_key, web_key.id(), &nonce).unwrap();

        assert_eq!(dec_key, web_key)
    }
}
