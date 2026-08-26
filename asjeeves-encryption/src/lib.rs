//! Encryption utilities
//!
//! # Randomization
//!
//! ## Examples
//! You can generate one time values (nonces). This
//! uses the UUID V7 spec. See: [RFC9562](https://datatracker.ietf.org/doc/rfc9562/)
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     let nonce : Box<str> = generate_nonce();
//! ```
//! You can generate a ChaCha 20 randomizer via a seed see [wikipedia](https://en.wikipedia.org/wiki/Salsa20#ChaCha_variant).
//! [seed::Rng] implements [rand_core::CryptoRng], [rand_core::RngCore], and
//! [rand_core::SeedableRng] and thus can be used with many libraries that require rng.
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     let seed = Seed::default();
//!
//!     let mut rng = seed.rng();
//! ```
//!
//! # Private Keys
//!
//! ## Examples
//! You can generate keys.
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     let seed = Seed::default();
//!     let mut rng: Rng = seed.rng();
//!
//!     let key = PrivateKey::generate(&mut rng).unwrap();
//!     let kek = KeyEncryptionKey::generate(&mut rng);
//!
//! ```
//!
//! You can also load a key encryption key from existing data.
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     // You need to use a 256-bit (32 bytes) string.
//!     let s = String::from("12345678901234567890123456789012");
//!
//!     let kek = KeyEncryptionKey::try_from(s).unwrap();
//! ```
//!
//! You can encrypt and decrypt data
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     let seed = Seed::default();
//!     let mut rng = seed.rng();
//!
//!     let key = PrivateKey::generate(&mut rng).unwrap();
//!
//!     // Sensitive ensures that when data leaves scope it is zeroed out.
//!     let unenc = Sensitive::new(vec![1u8, 2u8].into_boxed_slice());
//!
//!     let enc = key.encrypt_bytes(unenc, &mut rng).unwrap();
//!
//!     let dec = key.decrypt_bytes(&enc).unwrap();
//! ```
//!
//! You can encrypt and decrypt [private_key::PrivateKey].
//! ```
//!     use asjeeves_encryption::prelude::*;
//!
//!     let seed = Seed::default();
//!     let mut rng: Rng = seed.rng();
//!
//!     let key = PrivateKey::generate(&mut rng).unwrap();
//!     let kek = KeyEncryptionKey::generate(&mut rng);
//!
//!     let enc_key : EncryptedKey = kek.encrypt_key(&key, &mut rng).unwrap();
//!
//!     let key : PrivateKey = kek.decrypt_key(&enc_key.key, &enc_key.key_id, &enc_key.nonce).unwrap();
//! ```
//!
//! You can verify
//! the signature of anything that implements
//! [serde::Deserialize] and [serde::Serialize].
//! ```
//!     use asjeeves_encryption::prelude::*;
//!     use serde::{Deserialize, Serialize};
//!
//!     #[derive(Debug, Deserialize, Serialize)]
//!     struct Claims {
//!         iat: u64,
//!         name: Box<str>,
//!         sub: Box<str>,
//!     }
//!
//!     let claims = Claims {
//!         iat: 12345667890,
//!         name: "name".into(),
//!         sub: "subject".into()
//!     };
//!
//!     let seed = Seed::default();
//!     let mut rng = seed.rng();
//!
//!     let key = PrivateKey::generate(&mut rng).unwrap();
//!
//!     let jws = key.sign_claims(claims).unwrap();
//!     let jwk = key.to_json_web_key();
//!
//!     let result = jwk.verify_signature(jws);
//!
//!     assert!(result.is_ok(), "Signature should be valid");
//!
//! ```

pub mod error;
pub mod kek;
pub mod nonce;
pub mod private_key;
pub mod seed;
pub mod sensitive;
pub mod signature_verification;

pub use error::Error;

pub mod jwt {
    pub use json_web_tolkien::*;
}

pub mod prelude {
    pub use super::signature_verification::SignatureVerification;
    pub use crate::kek::{EncryptedKey, KeyEncryptionKey};
    pub use crate::nonce::generate_nonce;
    pub use crate::private_key::PrivateKey;
    pub use crate::seed::{Rng, Seed};
    pub use crate::sensitive::Sensitive;
    pub use json_web_key::jwk::{JsonWebKey, JsonWebKeySet};
    pub use json_web_tolkien::prelude::{Algorithm, Jws, Jwt};
}
