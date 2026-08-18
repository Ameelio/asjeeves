use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

use json_web_key::prelude::*;
use json_web_tolkien::jws::Jws;
use json_web_tolkien::jwt::Jwt;
use json_web_tolkien::prelude::Algorithm;
use rand_core::{CryptoRng, RngCore};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, SecretDocument};
use rsa::sha2::Sha256;
use rsa::signature::{SignatureEncoding, SignerMut, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::error::Error;
use crate::prelude::Sensitive;

pub type WebKey = PrivateKey;

#[derive(Clone, PartialEq)]
pub struct PrivateKey {
    inner: Arc<RsaPrivateKey>,
    id: Arc<str>,
}

impl PrivateKey {
    #[instrument(skip(ciphertext))]
    pub fn decrypt_bytes(&self, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
        let dec_bytes: Vec<u8> = self
            .inner
            .decrypt(Pkcs1v15Encrypt, ciphertext)
            .map_err(|source| Error::DecryptionError { source })?;

        Ok(dec_bytes)
    }

    #[instrument]
    pub fn encrypt_bytes<R>(
        &self,
        bytes: Sensitive<Box<[u8]>>,
        rng: &mut R,
    ) -> Result<Vec<u8>, Error>
    where
        R: CryptoRng + RngCore + fmt::Debug,
    {
        let enc_bytes: Vec<u8> = self
            .pubkey()
            .encrypt(rng, Pkcs1v15Encrypt, &bytes)
            .map_err(|source| Error::EncryptionError { source })?;

        Ok(enc_bytes)
    }

    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    #[instrument]
    pub fn sign_claims<C>(&self, claims: C) -> Result<Jws, Error>
    where
        C: fmt::Debug + for<'de> Deserialize<'de> + Serialize,
    {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        struct Header {
            alg: Algorithm,
            kid: Box<str>,
            typ: Box<str>,
        }

        let header = Header {
            alg: Algorithm::RS256,
            kid: self.id.as_ref().into(),
            typ: "JWT".into(),
        };

        let token = Jwt::new(claims, header);

        self.sign_json_web_token(token)
    }

    #[instrument(skip(token))]
    pub fn sign_json_web_token<C, H>(&self, token: Jwt<C, H>) -> Result<Jws, Error>
    where
        C: fmt::Debug + Serialize,
        H: fmt::Debug + Serialize,
    {
        let mut signing_key: SigningKey<Sha256> = SigningKey::new((*self.inner).clone());

        let encoded_token: String = token
            .to_string()
            .map_err(|source| Error::JwtSerializationError { source })?;

        let signature: Box<[u8]> = {
            let msg: &[u8] = encoded_token.as_bytes();

            let signature: Signature = signing_key.sign(msg);

            signature.to_bytes()
        };

        let jws = Jws::new(encoded_token.as_str(), signature.as_ref());

        Ok(jws)
    }

    #[instrument]
    pub fn generate<R>(rng: &mut R) -> Result<Self, Error>
    where
        R: CryptoRng + RngCore + fmt::Debug,
    {
        const BITS: usize = 2048;

        let inner: Arc<RsaPrivateKey> = {
            let inner =
                RsaPrivateKey::new(rng, BITS).map_err(|source| Error::KeyGenError { source })?;

            Arc::new(inner)
        };

        let id: Arc<str> = {
            let id = Uuid::now_v7();
            let id = id.to_string();
            let id = id.as_str();

            id.into()
        };

        let key = Self { id, inner };

        Ok(key)
    }

    #[instrument]
    pub fn to_json_web_key(&self) -> JsonWebKey {
        let key_id: Box<str> = (*self.id.clone()).into();
        let pubkey = self.pubkey();

        let exponent: Box<[u8]> = pubkey.e().to_bytes_be().into();
        let modulus: Box<[u8]> = pubkey.n().to_bytes_be().into();

        let rwk = RsaWebKey {
            exponent,
            key_id,
            modulus,
            ..RsaWebKey::default()
        };

        JsonWebKey::RS256(rwk)
    }

    #[instrument]
    pub fn to_json(&self) -> Result<serde_json::Value, Error> {
        let jwk: JsonWebKey = self.to_json_web_key();

        let json_val: serde_json::Value = serde_json::to_value(jwk)?;

        Ok(json_val)
    }

    #[instrument(skip(jws))]
    pub fn verify_json_web_signature(&self, jws: &Jws) -> Result<(), Error> {
        let key: VerifyingKey<Sha256> = VerifyingKey::new(self.pubkey());

        let signature: Signature = {
            let sig: Cow<[u8]> = jws.signature();
            Signature::try_from(sig.as_ref())?
        };

        let msg: Cow<str> = jws.encoded_token();

        key.verify(msg.as_bytes(), &signature)?;

        Ok(())
    }

    pub(crate) fn from_bytes(bytes: Zeroizing<Vec<u8>>, id: Arc<str>) -> Result<Self, Error> {
        let inner: Arc<RsaPrivateKey> = {
            let inner = RsaPrivateKey::from_pkcs8_der(&bytes)?;

            Arc::new(inner)
        };

        let key = Self { id, inner };

        Ok(key)
    }

    pub(crate) fn to_bytes(&self) -> Result<Zeroizing<Vec<u8>>, Error> {
        let doc: SecretDocument = self.inner.to_pkcs8_der()?;

        let bytes: Zeroizing<Vec<u8>> = doc.to_bytes();

        Ok(bytes)
    }

    fn pubkey(&self) -> RsaPublicKey {
        RsaPublicKey::from(self.inner.as_ref())
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKey")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Seed;

    #[test]
    fn it_should_enc_and_decrypt() {
        let seed = Seed::from(1);
        let mut rng = seed.rng();

        let key = WebKey::generate(&mut rng).unwrap();

        let unenc = Sensitive::new(vec![1u8, 2u8].into_boxed_slice());

        let enc = key.encrypt_bytes(unenc, &mut rng).unwrap();

        let dec = key.decrypt_bytes(&enc).unwrap();

        assert_eq!(vec![1u8, 2u8], dec);
    }
}
