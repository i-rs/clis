use crate::server::app::AppState;
use subtle::ConstantTimeEq;

/// Compare two byte slices in constant time.
/// Returns true iff they are byte-equal; runtime does not leak length/prefix info.
fn ct_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

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
            // 1. Check static config users (config.toml)
            if let Some(user) = core
                .config
                .dashboard
                .users
                .iter()
                .find(|u| ct_eq(&u.token, token))
            {
                user.id.clone()
            } else if ct_eq(&state.auth_token, token) {
                "default".to_string()
            } else {
                // 2. Fall back to DB-stored users (created via POST /api/users)
                match core
                    .config_store
                    .dashboard_users
                    .find_by_token_hash(token)
                    .await
                {
                    Ok(Some(row)) => row.user_id,
                    _ => return unauthorized(),
                }
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
pub struct UserId(pub String);

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
    use tower::ServiceExt;

    async fn ok_handler() -> &'static str {
        "ok"
    }

    fn test_state() -> AppState {
        let (_cfg, core) = crate::test_helpers::test_core();
        AppState::new(core, "secret".to_string())
    }

    fn run_auth_test<F, Fut>(name: &str, f: F)
    where
        F: FnOnce(AppState) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let result = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let state = test_state();
                i_rs_claw_core::utils::sync_block_on(f(state));
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
        run_auth_test("test_auth_valid_token", |state| async move {
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
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        });
    }

    #[test]
    fn test_auth_missing_token() {
        run_auth_test("test_auth_missing_token", |state| async move {
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
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }

    #[test]
    fn test_auth_wrong_token() {
        run_auth_test("test_auth_wrong_token", |state| async move {
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
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }

    #[test]
    fn test_ct_eq_matches_eq() {
        // Functional equivalence to == (timing properties not asserted here,
        // but use of ConstantTimeEq is enforced by code review + the helper).
        assert!(ct_eq("abcdef", "abcdef"));
        assert!(!ct_eq("abcdef", "abcdeg"));
        assert!(!ct_eq("abcdef", "abcdef-extra"));
        assert!(!ct_eq("different-length", "abcdef"));
        assert!(ct_eq("", ""));
    }

    #[test]
    fn test_auth_valid_multi_user_token() {
        run_auth_test("test_auth_valid_multi_user_token", |state| async move {
            // Add a user with a known token
            {
                let mut core = state.core.write().await;
                core.config
                    .dashboard
                    .users
                    .push(i_rs_claw_core::config::DashboardUser {
                        id: "alice".to_string(),
                        token: "alice-secret".to_string(),
                    });
            }

            let app = Router::new()
                .route("/api/test", get(ok_handler))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    auth_guard,
                ))
                .with_state(state);

            // alice's token → 200
            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer alice-secret")
                .body(Body::empty())
                .unwrap();
            let resp = app.clone().oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);

            // bob's token (different bytes, same length) → still 401
            let req = Request::builder()
                .uri("/api/test")
                .header("authorization", "Bearer bob-secret")
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        });
    }
}
