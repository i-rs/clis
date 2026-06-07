use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::Value;

/// GET /api/users — List all dashboard users.
pub async fn list_users(
    State(state): State<AppState>,
) -> Json<super::ApiResponse<Vec<Value>>> {
    let core = state.core.read().await;
    match core.config_store.dashboard_users.load_all().await {
        Ok(users) => {
            let result: Vec<Value> = users
                .iter()
                .map(|u| {
                    serde_json::json!({
                        "user_id": u.user_id,
                        "display_name": u.display_name,
                        "created_at": u.created_at,
                    })
                })
                .collect();
            super::ApiResponse::ok(result)
        }
        Err(e) => super::ApiResponse::err(&format!("Failed to list users: {}", e)),
    }
}

/// POST /api/users — Create a new dashboard user.
pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<super::ApiResponse<Value>> {
    let user_id = match body.get("user_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => return super::ApiResponse::err("Missing or invalid 'user_id' field"),
    };

    let token = match body.get("token").and_then(|v| v.as_str()) {
        Some(t) if !t.is_empty() => t.to_string(),
        _ => uuid::Uuid::new_v4().to_string(),
    };

    let display_name = body
        .get("display_name")
        .and_then(|v| v.as_str())
        .unwrap_or(&user_id)
        .to_string();

    let now = chrono::Utc::now().timestamp();

    let core = state.core.write().await;

    let row = i_rs_claw_core::storage::config_store::DashboardUserRow {
        user_id: user_id.clone(),
        token_hash: token.clone(),
        display_name: display_name.clone(),
        created_at: now,
        updated_at: now,
    };

    match core.config_store.dashboard_users.upsert(&row).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "user_id": user_id,
            "token": token,
            "display_name": display_name,
            "status": "created",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to create user: {}", e)),
    }
}

/// DELETE /api/users/{id} — Delete a dashboard user.
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.write().await;

    match core.config_store.dashboard_users.delete(&user_id).await {
        Ok(_) => super::ApiResponse::ok(serde_json::json!({
            "user_id": user_id,
            "status": "deleted",
        })),
        Err(e) => super::ApiResponse::err(&format!("Failed to delete user: {}", e)),
    }
}
