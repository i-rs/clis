use crate::server::AppState;
use axum::{
    Json,
    extract::{Query, State},
};
use i_rs_claw_core::stats::StatsPeriod;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct StatsQuery {
    #[serde(default = "default_period")]
    pub period: String,
}

fn default_period() -> String {
    "today".to_string()
}

/// Get token usage statistics.
/// Query params: ?period=today|7d|30d|all (default: today)
pub async fn get_stats(
    State(state): State<AppState>,
    Query(query): Query<StatsQuery>,
) -> Json<super::ApiResponse<Value>> {
    let core = state.core.read().await;

    if !core.config.stats.enabled {
        return super::ApiResponse::err("Token usage statistics are disabled");
    }

    let period = match query.period.as_str() {
        "7d" | "7days" => StatsPeriod::Last7Days,
        "30d" | "30days" => StatsPeriod::Last30Days,
        "all" => StatsPeriod::All,
        _ => StatsPeriod::Today,
    };

    let result = core.stats_manager.query_async(period).await;
    let today = core.stats_manager.today_summary_async().await;

    match serde_json::to_value(&result) {
        Ok(mut v) => {
            if let Some(obj) = v.as_object_mut() {
                obj.insert(
                    "today".to_string(),
                    serde_json::json!({
                        "requests": today.requests,
                        "tokens": today.tokens,
                        "cost_usd": today.cost_usd,
                    }),
                );
            }
            super::ApiResponse::ok(v)
        }
        Err(e) => super::ApiResponse::err(&format!("Failed to serialize stats: {}", e)),
    }
}
