use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("an unexpected decryption error, {source}")]
    AesDecryptionError { source: aes_gcm::Error },
    #[error("an unexpected decryption error, {source}")]
    DecryptionError { source: rsa::Error },
    #[error("an unexpected encryption error, {source}")]
    AesEncryptionError { source: aes_gcm::Error },
    #[error("an unexpected encryption error, {source}")]
    EncryptionError { source: rsa::Error },
    #[error("invalid key format, {source}")]
    InvalidKeyFormat {
        #[from]
        source: rsa::pkcs8::Error,
    },
    #[error("invalid signature, {source}")]
    InvalidSignature {
        #[from]
        source: rsa::signature::Error,
    },
    #[error("unable to serialize jwk, {source}")]
    JwkSerializationError {
        #[from]
        source: serde_json::Error,
    },
    #[error("unable to serialize token, {source}")]
    JwtSerializationError {
        source: json_web_tolkien::error::Error,
    },
    #[error("unable to generate key, {source}")]
    KeyGenError { source: rsa::Error },
    #[error("unsupported algorithm")]
    UnsupportedAlgorithm,
}
