use crate::server::app::AppState;

/// Axum middleware that validates Bearer token and resolves user_id.
pub async fn auth_guard(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let provided = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let user_id: String = match provided {
        Some(token) => {
            let core = state.core.read().await;
            if let Some(user) = core.config.dashboard.users.iter().find(|u| u.token == token) {
                user.id.clone()
            } else if state.auth_token == token {
                "default".to_string()
            } else {
                return unauthorized();
            }
        }
        None => return unauthorized(),
    };

    req.extensions_mut().insert(UserId(user_id));
    next.run(req).await
}

fn unauthorized() -> axum::response::Response {
    let mut resp = axum::response::IntoResponse::into_response(axum::Json(
        serde_json::json!({"success": false, "data": null, "error": "Unauthorized"}),
    ));
    *resp.status_mut() = axum::http::StatusCode::UNAUTHORIZED;
    resp.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_static("Bearer realm=\"claw-dashboard\""),
    );
    resp
}

/// Extractor: resolves the current user_id from request context.
#[derive(Clone, Debug)]
pub struct UserId(#[allow(dead_code)] pub String);

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for UserId {
    type Rejection = (axum::http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<UserId>()
            .cloned()
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing user context"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState {
            core: std::sync::Arc::new(RwLock::new(core)),
            auth_token: "secret".to_string(),
        }
    }

    fn run_auth_test<F>(name: &str, f: F)
    where
        F: FnOnce(tokio::runtime::Runtime, AppState) + Send + 'static,
    {
        let result = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("create test runtime");
                let state = test_state();
                f(rt, state);
            })
            .expect("spawn test thread")
            .join();
        if let Err(e) = result {
            if let Some(msg) = e.downcast_ref::<&str>() {
                panic!("test '{}' panicked: {}", name, msg);
            } else if let Some(msg) = e.downcast_ref::<String>() {
                panic!("test '{}' panicked: {}", name, msg);
            } else {
                panic!("test '{}' panicked (unknown)", name);
            }
        }
    }

    #[test]
    fn test_auth_valid_token() {
        run_auth_test("test_auth_valid_token", |rt, state| {
            let app = Router::new()
                .route("/api/test", get(ok_handler))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_guard,
                ))
                .with_state(state);

            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer secret")
                .body(Body::empty())
                .unwrap();
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        });
    }

    #[test]
    fn test_auth_missing_token() {
        run_auth_test("test_auth_missing_token", |rt, state| {
            let app = Router::new()
                .route("/api/test", get(ok_handler))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_guard,
                ))
                .with_state(state);

            let req = Request::builder()
                .uri("/api/test")
                .body(Body::empty())
                .unwrap();
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }

    #[test]
    fn test_auth_wrong_token() {
        run_auth_test("test_auth_wrong_token", |rt, state| {
            let app = Router::new()
                .route("/api/test", get(ok_handler))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_guard,
                ))
                .with_state(state);

            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer wrong")
                .body(Body::empty())
                .unwrap();
            let resp = rt.block_on(app.oneshot(req)).unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }
}
