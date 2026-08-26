use std::pin::Pin;
use std::task::{Context, Poll};

use axum::body::Body;
use axum::extract::Request;
use axum::http::Response;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use tower::Service;

use crate::store_user_session::store_user_session;
use crate::user_session::UserSession;
use crate::user_session_state::UserSessionState;

/// [tower] middleware that can:
/// - Provide a [UserSession] if a cookie exists, it can then be extracted
///   by the inner [Service].
/// - Add a session cookie if [UserSession] returned by an inner [Service] such as
///   an axum handler.
#[derive(Clone)]
pub struct UserSessionService<InnerService> {
    pub(crate) inner: InnerService,
    pub(crate) state: UserSessionState,
}

impl<InnerService, B> Service<Request<B>> for UserSessionService<InnerService>
where
    InnerService: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    InnerService::Future: Send + 'static,
{
    type Response = InnerService::Response;
    type Error = InnerService::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        req.extensions_mut().insert(self.state.clone());

        let future = self.inner.call(req);

        let state = self.state.clone();
        Box::pin(async move {
            let mut response = future.await?;

            // If a handler puts UserSession as part of the response, it will be available
            // here. We want to handle most requests where we DO NOT need to create a
            // new UserSession.
            let Some(UserSession(user_session)): Option<UserSession<serde_json::Value>> =
                response.extensions_mut().remove()
            else {
                return Ok(response);
            };

            // But when we DO have a user_session, we want to store it in redis, and then
            // propogate the cookie.
            match store_user_session(&state, user_session).await {
                Ok(cookie_header) => {
                    // http spec requires if we have multiple cookies to have a header
                    // for each SET_COOKIE, this means we can skip the cookiejar, there
                    // is a small risk something else sets our user_session cookie or
                    // one with the same name.
                    response.headers_mut().append(SET_COOKIE, cookie_header);

                    Ok(response)
                }
                Err(e) => Ok(e.into_response()),
            }
        })
    }

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
}
