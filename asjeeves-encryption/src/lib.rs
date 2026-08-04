pub mod error;
pub mod kek;
pub mod nonce;
pub mod seed;
pub mod sensitive;
pub mod signature_verification;
pub mod web_key;

pub use error::Error;

pub mod jwt {
    pub use json_web_tolkien::*;
}

pub mod prelude {
    pub use super::signature_verification::SignatureVerification;
    pub use crate::kek::{EncryptedKey, KeyEncryptionKey};
    pub use crate::nonce::generate_nonce;
    pub use crate::seed::{Rng, Seed};
    pub use crate::sensitive::Sensitive;
    pub use crate::web_key::{PrivateKey, WebKey};
    pub use json_web_key::jwk::{JsonWebKey, JsonWebKeySet};
    pub use json_web_tolkien::prelude::{Algorithm, Jws, Jwt};
}
