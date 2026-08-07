//! CSRF middleware
//!     - Adds a CSRF cookie on GET requests
//!     - Execpects a CSRF cookie and matches it on DELETE/POST/PUT requests.
//!
//! ## Setup
//!     - Add protect_from_forgery using `axum::middleware::from_fn_with_state`.
//!     - Implement `FromRef<Rng>` for your state.

use asjeeves_encryption::seed::Rng;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use tracing::instrument;

use crate::form_authenticity_token::FormAuthenticityToken;

#[instrument(err, skip(cookie_fat))]
pub async fn protect_against_forgery(
    State(rng): State<Rng>,
    cookie_fat: Option<FormAuthenticityToken>,
    headers: HeaderMap,
    method: Method,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match method {
        Method::GET => {
            let mut response: Response = next.run(request).await;

            let cookie: String = {
                let mut rng = rng;

                let fat = FormAuthenticityToken::generate(&mut rng);

                fat.csrf_cookie().to_string()
            };

            if let Ok(hdr_val) = HeaderValue::from_str(&cookie) {
                response.headers_mut().insert(header::SET_COOKIE, hdr_val);
            }

            Ok(response)
        }
        Method::PUT | Method::POST | Method::DELETE => {
            let cookie_fat: FormAuthenticityToken = cookie_fat.ok_or(StatusCode::FORBIDDEN)?;

            let client_fat: FormAuthenticityToken =
                fetch_client_fat(&headers).ok_or(StatusCode::FORBIDDEN)?;

            if client_fat == cookie_fat {
                let response: Response = next.run(request).await;

                Ok(response)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        _ => {
            let response: Response = next.run(request).await;

            Ok(response)
        }
    }
}

const X_CSRF_TOKEN: &str = "X-CSRF-TOKEN";

fn fetch_client_fat(headers: &HeaderMap) -> Option<FormAuthenticityToken> {
    if headers.contains_key(X_CSRF_TOKEN) {
        let hdr: HeaderValue = headers[X_CSRF_TOKEN].clone();

        let fat: FormAuthenticityToken = hdr.into();

        return Some(fat);
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::form_authenticity_token::COOKIE_NAME;
    use crate::form_authenticity_token::test::{FAT_ONE, FAT_TWO};
    use asjeeves_encryption::seed::{Rng, Seed};
    use axum::extract::FromRef;
    use axum::http::HeaderValue;
    use axum::middleware;
    use axum::routing::{Router, put};
    use axum_extra::extract::cookie::Cookie;
    use axum_test::TestServer;

    async fn test_handler() -> StatusCode {
        StatusCode::OK
    }

    #[derive(Clone, Default, Debug)]
    struct State {
        seed: Seed,
    }

    impl FromRef<State> for Rng {
        fn from_ref(input: &State) -> Self {
            input.seed.rng()
        }
    }

    #[tokio::test]
    async fn it_should_continue_the_request_if_valid() {
        let state = State::default();

        let app =
            Router::new()
                .route("/", put(test_handler))
                .layer(middleware::from_fn_with_state(
                    state,
                    protect_against_forgery,
                ));

        let server = TestServer::new(app).unwrap();

        let client_csrf = HeaderValue::from_static(FAT_ONE);
        let cookie = Cookie::new(COOKIE_NAME, FAT_ONE);

        let response = server
            .put("/")
            .add_header(X_CSRF_TOKEN, client_csrf)
            .add_cookie(cookie)
            .await;

        response.assert_status_ok();
    }

    #[tokio::test]
    async fn it_should_halt_the_request_if_invalid() {
        let state = State::default();

        let app =
            Router::new()
                .route("/", put(test_handler))
                .layer(middleware::from_fn_with_state(
                    state,
                    protect_against_forgery,
                ));

        let server = TestServer::new(app).unwrap();

        let client_csrf = HeaderValue::from_static(FAT_ONE);
        let cookie = Cookie::new(COOKIE_NAME, FAT_TWO);

        let response = server
            .put("/")
            .add_header(X_CSRF_TOKEN, client_csrf)
            .add_cookie(cookie)
            .await;

        response.assert_status_forbidden();
    }
}
