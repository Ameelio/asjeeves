//! # Form Authenticity Token
//! A randomized base64 string useable for CSRF protection.
//! See: [Cross-site request forgery](https://en.wikipedia.org/wiki/Cross-site_request_forgery)
//!
//! ## Randomization used
//! As of 2025.07.17 The from_entropy() function of the rand crate behaves as such:
//!     - Automatic seeding and reseeding via OsRng (for Linux this is the getrandom syscall, or /dev/urandom)
//!     - Algorithm used: ChaCha (20 rounds)
//!     - Does not zero memory on exit. (No protection for internal memory state)

use std::{borrow::Cow, convert::Infallible, fmt};

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::HeaderValue,
};
use axum_extra::extract::CookieJar;
use base64ct::{Base64Url, Encoding};
use cookie::{Cookie, SameSite};

use rand_core::RngCore;

use tracing::instrument;

pub const COOKIE_NAME: &str = "csrf_token";

// Recommended length (128 bits / 16 bytes) for the token.
const TOKEN_LEN: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct FormAuthenticityToken(Box<str>);

impl FormAuthenticityToken {
    /// Generates a random token.
    /// # Examples
    ///
    /// ```
    /// use asjeeves_csrf::FormAuthenticityToken;
    /// use rand_chacha::ChaCha20Rng;
    /// use rand_core::SeedableRng;
    ///
    /// let mut rng = ChaCha20Rng::from_seed(Default::default());
    ///
    /// let fat = FormAuthenticityToken::generate(&mut rng);
    ///
    /// ```
    #[instrument]
    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: fmt::Debug + RngCore,
    {
        // Allocate a 32 byte array.
        let mut token_bytes = [0u8; TOKEN_LEN];

        rng.fill_bytes(&mut token_bytes);

        let token = Base64Url::encode_string(&token_bytes);

        Self(token.into_boxed_str())
    }

    /// Returns a cookie with the token formatted for csrf
    /// Secure; SameSite=Strict; HttpOnly; Path=/
    pub fn csrf_cookie<'a>(&self) -> Cookie<'a> {
        Cookie::build((COOKIE_NAME, self.to_string()))
            .http_only(true)
            .partitioned(true)
            .same_site(SameSite::Strict)
            .secure(true)
            .path("/")
            .build()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<str> for FormAuthenticityToken {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl fmt::Display for FormAuthenticityToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.as_ref())
    }
}

impl<'a> From<&Cookie<'a>> for FormAuthenticityToken {
    fn from(c: &Cookie<'a>) -> Self {
        let s = String::from(c.value());

        Self(s.into_boxed_str())
    }
}

impl From<HeaderValue> for FormAuthenticityToken {
    fn from(value: HeaderValue) -> Self {
        let bytes: &[u8] = value.as_bytes();

        let s: Cow<'_, str> = String::from_utf8_lossy(bytes);

        Self(s.into())
    }
}

impl<S> OptionalFromRequestParts<S> for FormAuthenticityToken
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        let jar: CookieJar = CookieJar::from_request_parts(parts, state).await?;
        let fat: Option<Self> = jar.get(COOKIE_NAME).map(Self::from);

        Ok(fat)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;

    pub const FAT_ONE: &str = "mjdEUEVgY57GcLehfUkrJz4HewqWvvWLp3YHeeVEVG4=";
    pub const FAT_TWO: &str = "AA7-yHxXSewRV5EuDhcfYN6eU0E0iBmi3pnxQMWaQkw=";

    #[test]
    fn it_should_create_a_token_and_cookie() {
        let mut rng = ChaCha20Rng::seed_from_u64(1);

        let fat_one = FormAuthenticityToken::generate(&mut rng);
        let fat_two = FormAuthenticityToken::generate(&mut rng);

        let ec_one = format!(
            "{}={}; HttpOnly; SameSite=Strict; Partitioned; Secure; Path=/",
            COOKIE_NAME, FAT_ONE
        );
        let ec_two = format!(
            "{}={}; HttpOnly; SameSite=Strict; Partitioned; Secure; Path=/",
            COOKIE_NAME, FAT_TWO
        );

        assert_eq!(FAT_ONE, fat_one.as_str());
        assert_eq!(FAT_ONE, fat_one.as_ref());
        assert_eq!(ec_one, fat_one.csrf_cookie().to_string());

        assert_eq!(FAT_TWO, fat_two.as_str());
        assert_eq!(FAT_TWO, fat_two.as_ref());
        assert_eq!(ec_two, fat_two.csrf_cookie().to_string());
    }
}
