//! Prevents XSS attacks.
//! For information on XSS see [Wikipedia](https://en.wikipedia.org/wiki/Cross-site_scripting)
//!
//! ## Example
//! You add the middleware as any other [axum] layer.
//!
//! ```
//! use axum::middleware;
//! use axum::routing::Router;
//!
//! use asjeeves_csrf::middleware::protect_against_forgery;
//!
//! #[derive(Clone)]
//! struct WebState {}
//!
//! let csrf = middleware::from_fn(protect_against_forgery);
//!
//! let app : Router<WebState> = Router::new()
//!     .layer(csrf)
//!     .with_state(WebState {});
//!
//! ```
//!
//! You will need to supply the initial CSRF token and store it on requests
//! rendering a form.
//! ```
//! use asjeeves_csrf::FormAuthenticityToken;
//! use asjeeves_encryption::prelude::*;
//! use axum::extract::State;
//!
//! #[derive(Clone)]
//! struct WebState {
//!     seed: Seed,
//! }
//!
//! pub async fn handler(State(state): State<WebState>) -> FormAuthenticityToken {
//!     let mut rng = state.seed.rng();
//!
//!     FormAuthenticityToken::generate(&mut rng)
//! }
//! ```

mod form_authenticity_token;
pub mod middleware;

pub use form_authenticity_token::FormAuthenticityToken;
