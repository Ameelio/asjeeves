use std::borrow::Cow;

use json_web_key::prelude::*;
use json_web_tolkien::prelude::*;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::sha2::Sha256;
use rsa::signature::Verifier;
use rsa::{BigUint, RsaPublicKey};

use crate::Error;

pub trait SignatureVerification {
    fn verify_signature(&self, jws: Jws) -> Result<(), Error>;
}

impl SignatureVerification for JsonWebKey {
    fn verify_signature(&self, jws: Jws) -> Result<(), Error> {
        let JsonWebKey::RS256(key) = self else {
            return Err(Error::UnsupportedAlgorithm);
        };

        let exponent = BigUint::from_bytes_be(key.exponent.as_ref());
        let modulus = BigUint::from_bytes_be(key.modulus.as_ref());

        let key =
            RsaPublicKey::new(modulus, exponent).map_err(|source| Error::KeyGenError { source })?;

        let key = VerifyingKey::<Sha256>::new(key);

        let signature: Cow<[u8]> = jws.signature();
        let signature = Signature::try_from(signature.as_ref())?;

        let token: Cow<str> = jws.encoded_token();
        let token: &[u8] = token.as_bytes();

        key.verify(token, &signature)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Seed;
    use crate::web_key::WebKey;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1v15::SigningKey;
    use rsa::sha2::Sha256;
    use rsa::signature::{SignatureEncoding, Signer};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    struct Header {
        pub alg: Box<str>,
        pub kid: Box<str>,
    }

    #[derive(Deserialize, Serialize)]
    struct Claims {
        pub iat: u64,
        pub name: Box<str>,
        pub sub: Box<str>,
    }

    fn create_test_key_and_jwk() -> (RsaPrivateKey, JsonWebKey) {
        let seed = Seed::from(1);
        let mut rng = seed.rng();
        let web_key = WebKey::generate(&mut rng).expect("Failed to generate web key");
        let jwk = web_key.to_json_web_key();

        // Extract private key for signing
        let private_key = {
            let seed = Seed::from(1);
            let mut rng = seed.rng();
            RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key")
        };

        (private_key, jwk)
    }

    fn create_signed_jws(private_key: &RsaPrivateKey, claims: Claims) -> Jws {
        let header = Header {
            alg: "RS256".into(),
            kid: "test-key-id".into(),
        };

        let token: Jwt<Header, Claims> = Jwt::new(header, claims);

        let token: String = serde_json::to_string(&token).expect("jwt to serialize to a string");

        let signing_key = SigningKey::<Sha256>::new(private_key.clone());
        let signature = signing_key.sign(token.as_bytes());

        let jws = Jws::new(token.as_str(), signature.to_bytes().as_ref());

        jws
    }

    #[test]
    fn test_verify_valid_signature() {
        let (private_key, jwk) = create_test_key_and_jwk();
        let payload = Claims {
            sub: "1234567890".into(),
            name: "Test User".into(),
            iat: 1516239022,
        };
        let jws = create_signed_jws(&private_key, payload);

        let result = jwk.verify_signature(jws);
        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_verify_invalid_signature() {
        let (_, jwk) = create_test_key_and_jwk();

        // Create a different key for signing
        let seed = Seed::from(2);
        let mut rng = seed.rng();
        let wrong_private_key =
            RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate wrong key");

        let payload = Claims {
            sub: "1234567890".into(),
            name: "Test User".into(),
            iat: 1516239022,
        };

        let jws = create_signed_jws(&wrong_private_key, payload);

        let result = jwk.verify_signature(jws);
        assert!(
            result.is_err(),
            "Invalid signature should fail verification"
        );

        match result {
            Err(Error::InvalidSignature { .. }) => {}
            _ => panic!("Expected InvalidSignature error"),
        }
    }

    #[test]
    fn test_verify_tampered_payload() {
        let (private_key, jwk) = create_test_key_and_jwk();

        let original_payload = Claims {
            sub: "1234567890".into(),
            name: "Test User".into(),
            iat: 1516239022,
        };

        let jws = create_signed_jws(&private_key, original_payload);
        let signature: Cow<[u8]> = jws.signature();

        // Manually tamper with the JWS by modifying the payload part

        let tampered_payload = Claims {
            sub: "9999999999".into(),
            name: "Hacker".into(),
            iat: 1516239022,
        };

        let tampered_jws = create_signed_jws(&private_key, tampered_payload);

        let tampered_token = tampered_jws.encoded_token();

        let tampered_jws = Jws::new(tampered_token.as_ref(), signature.as_ref());

        let result = jwk.verify_signature(tampered_jws);
        assert!(result.is_err(), "Tampered payload should fail verification");

        match result {
            Err(Error::InvalidSignature { .. }) => {}
            _ => panic!("Expected InvalidSignature error for tampered payload"),
        }
    }
}
