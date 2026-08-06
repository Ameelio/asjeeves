//! Prevents XSS attacks.
//! For information on XSS see [Wikipedia](https://en.wikipedia.org/wiki/Cross-site_scripting)

mod form_authenticity_token;
pub mod middleware;

pub use form_authenticity_token::FormAuthenticityToken;
