use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;

use crate::form_authenticity_token::{COOKIE_NAME, FormAuthenticityToken};

pub async fn protect_against_forgery(
    headers: HeaderMap,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cookie_fat: FormAuthenticityToken =
        jar.get(COOKIE_NAME).ok_or(StatusCode::FORBIDDEN)?.into();

    let client_fat: FormAuthenticityToken =
        fetch_client_fat(&headers).ok_or(StatusCode::FORBIDDEN)?;

    if client_fat == cookie_fat {
        let response: Response = next.run(request).await;

        Ok(response)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

const X_CSRF_TOKEN: &'static str = "X-CSRF-TOKEN";

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
    use crate::form_authenticity_token::test::{FAT_ONE, FAT_TWO};
    use axum::http::HeaderValue;
    use axum::middleware;
    use axum::routing::{Router, put};
    use axum_extra::extract::cookie::Cookie;
    use axum_test::TestServer;

    async fn test_handler() -> StatusCode {
        StatusCode::OK
    }

    #[tokio::test]
    async fn it_should_continue_the_request_if_valid() {
        let app = Router::new()
            .route("/", put(test_handler))
            .layer(middleware::from_fn(protect_against_forgery));

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
        let app = Router::new()
            .route("/", put(test_handler))
            .layer(middleware::from_fn(protect_against_forgery));

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
