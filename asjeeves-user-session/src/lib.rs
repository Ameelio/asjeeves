//! # User Sessions
//! Middlware that handles extraction and storing of user session data.
//!
//! ## Example
//!     use std::env;
//!     use std::sync::Arc;
//!
//!     use axum::Router;
//!     use axum::body::Body;
//!     use axum::extract::{FromRef, State};
//!     use axum::http::{Request, Response, StatusCode};
//!     use axum::middleware::Next;
//!     use axum::response::Html;
//!     use axum::routing::{get, post};
//!     use chrono::TimeDelta;
//!     use redis::aio::ConnectionManager;
//!     use serde::{Deserialize, Serialize};
//!
//!     use asjeeves_encryption::prelude::*;
//!     use asjeeves_user_session::{UserSession, UserSessionKeys, UserSessionOptions, UserSessionLayer};
//!
//!     #[axum::debug_handler]
//!     async fn post_session(
//!         State(state): State<HttpState>,
//!     ) -> (UserSession<User>, StatusCode) {
//!         let inner = User {
//!             user_name: "montoya".into(),
//!         };
//!
//!         let session = UserSession(inner);
//!
//!         (session, StatusCode::CREATED)
//!     }
//!
//!     #[axum::debug_handler]
//!     async fn get_root(
//!         UserSession(session): UserSession<User>,
//!         // this is needed so that UserSession can extract the cache connection
//!         // from the state
//!         axum::extract::State(_state): axum::extract::State<HttpState>,
//!     ) -> (StatusCode, Html<String>) {
//!         let body = Html(format!("<p>{}</p>", session.user_name));
//!
//!         (StatusCode::OK, body)
//!     }
//!
//!     async fn handle_user_session_errors(req: Request<Body>, next: Next) -> Response<Body> {
//!         let mut response = next.run(req).await;
//!
//!         let err: Option<asjeeves_user_session::error::ErrorResponse> =
//!         response.extensions_mut().remove();
//!
//!         let Some(err) = err else {
//!             return response;
//!         };
//!
//!         let message: Body = err.to_string().into();
//!
//!         response.map(|_| message)
//!     }
//!
//!     #[tokio::main]
//!     async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!         let state: HttpState = {
//!             let cache : ConnectionManager = {
//!
//!                 let redis_url = env::var("TEST_REDIS_URL")?;
//!                 let client = redis::Client::open(redis_url)?;
//!
//!                 ConnectionManager::new(client).await?
//!             };
//!
//!             let seed = Seed::default();
//!
//!             HttpState { cache, seed }
//!         };
//!
//!         let usl : UserSessionLayer  = {
//!             let cache = state.cache.clone();
//!
//!             let options = UserSessionOptions::default();
//!
//!             let private_keys : UserSessionKeys  = {
//!                 let mut rng = state.seed.rng();
//!
//!                 let key : PrivateKey = PrivateKey::generate(&mut rng)?;
//!
//!                 let keys = vec![key];
//!
//!                 let keys : Arc<[PrivateKey]> = keys.into();
//!
//!                 UserSessionKeys(keys)
//!             };
//!
//!             let ttl = TimeDelta::days(90);
//!
//!             UserSessionLayer {
//!                 cache,
//!                 options,
//!                 private_keys,
//!                 ttl
//!             }
//!         };
//!
//!         let app : Router<HttpState> = Router::new()
//!             .route("/sessions", post(post_session))
//!             .route("/", get(get_root))
//!             .layer(usl)
//!             .layer(axum::middleware::from_fn(handle_user_session_errors))
//!             .with_state(state);
//!
//!         Ok(())
//!     }
//!
//!     #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
//!     struct User {
//!        pub user_name: Box<str>,
//!     }
//!
//!     #[derive(Clone, FromRef)]
//!     struct HttpState {
//!         cache: redis::aio::ConnectionManager,
//!         seed: Seed,
//!     }
pub mod error;
pub mod user_session_service;

mod fetch_user_session;
mod generate_session_key;
mod store_user_session;
mod user_session;
mod user_session_keys;
mod user_session_layer;
mod user_session_options;
mod user_session_state;

pub use user_session::UserSession;
pub use user_session_keys::UserSessionKeys;
pub use user_session_layer::UserSessionLayer;
pub use user_session_options::UserSessionOptions;

pub mod prelude {
    pub use super::{UserSession, UserSessionKeys, UserSessionLayer, UserSessionOptions};
}

#[cfg(test)]
pub mod tests;

#[cfg(test)]
mod test {
    use axum::Router;
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::Html;
    use axum::routing::{get, post};
    use axum_test::TestServer;
    use chrono::TimeDelta;
    use serde::{Deserialize, Serialize};

    use crate::tests::*;
    use crate::{UserSession, UserSessionLayer, UserSessionOptions};

    #[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
    struct MockInner {
        pub name: Box<str>,
    }

    #[tokio::test]
    async fn it_should_extract_a_user_session() {
        let state: TestState = test_state().await;

        #[axum::debug_handler]
        async fn post_session(
            State(_state): State<TestState>,
        ) -> (StatusCode, UserSession<MockInner>) {
            let inner = MockInner {
                name: "montoya".into(),
            };

            let session = UserSession(inner);

            (StatusCode::CREATED, session)
        }

        #[axum::debug_handler]
        async fn get_root(
            UserSession(session): UserSession<MockInner>,
            State(_state): State<TestState>,
        ) -> (StatusCode, Html<String>) {
            let body = Html(format!("<p>{}</p>", session.name));

            (StatusCode::OK, body)
        }

        let usl = UserSessionLayer {
            cache: state.cache.clone(),
            options: UserSessionOptions::default(),
            private_keys: state.keys.clone(),
            ttl: TimeDelta::minutes(3),
        };

        let router = Router::new()
            .route("/sessions", post(post_session))
            .route("/", get(get_root))
            .layer(usl)
            .layer(axum::middleware::from_fn(error_response_middleware))
            .with_state(state);

        let server = TestServer::builder().save_cookies().build(router).unwrap();

        let response = server.post("/sessions").await;

        response.assert_text("");
        response.assert_status(StatusCode::CREATED);
        // this panics if cookie is not present.
        let _ = response.cookie("user-session-id");

        let response = server.get("/").await;

        response.assert_text("<p>montoya</p>");
        response.assert_status_ok();
    }
}
