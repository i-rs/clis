mod api;
mod response;
mod routes;
mod store;

use axum::{
    http::{Method, header},
};
use tower_http::cors::{CorsLayer, Any};

use store::SharedStore;

// ═══════════════════════════════════════════════════════════════
// Macro: generate AppState + load_state + build_base_router from
// a single list of (field_name, store_type, filename) tuples.
// Also generates per-tool data-export/import/clear handlers.
// ═══════════════════════════════════════════════════════════════

macro_rules! make_app_tools {
    ($($field:ident: $store:ty => $file:expr),* $(,)?) => {
        paste::paste! {
            pub struct AppState {
                $(pub $field: SharedStore<$store>),*
            }

            const API_ENDPOINTS: &[&str] = &[$(concat!("/api/", stringify!($field))),*];

            fn load_state() -> std::sync::Arc<AppState> {
                std::sync::Arc::new(AppState {
                    $($field: SharedStore::load($file)),*
                })
            }

            fn build_base_router(
                state: std::sync::Arc<AppState>,
            ) -> axum::Router {
                async fn health_handler() -> axum::Json<serde_json::Value> {
                    axum::Json(serde_json::json!({
                        "success": true,
                        "status": "ok",
                        "service": "i-rs-api",
                        "version": env!("CARGO_PKG_VERSION"),
                        "description": "REST API for i-rs CLI tools",
                        "endpoints": API_ENDPOINTS
                    }))
                }

                // Per-tool data-export/import/clear handlers
                $(
                    async fn [<export_ $field _data>](
                        axum::extract::State(state):
                            axum::extract::State<std::sync::Arc<AppState>>,
                    ) -> crate::response::ApiResult<axum::Json<serde_json::Value>> {
                        let json = state.$field.export_json();
                        let data: serde_json::Value =
                            serde_json::from_str(&json).unwrap_or_default();
                        Ok(crate::api::ok_json(data))
                    }

                    async fn [<import_ $field _data>](
                        axum::extract::State(state):
                            axum::extract::State<std::sync::Arc<AppState>>,
                        body: String,
                    ) -> crate::response::ApiResult<axum::Json<serde_json::Value>> {
                        state
                            .$field
                            .import_json(&body)
                            .map_err(|e| crate::response::ApiError::BadRequest(e))?;
                        Ok(crate::api::ok_json_message())
                    }

                    async fn [<clear_ $field _data>](
                        axum::extract::State(state):
                            axum::extract::State<std::sync::Arc<AppState>>,
                    ) -> crate::response::ApiResult<axum::Json<serde_json::Value>> {
                        state.$field.clear();
                        Ok(crate::api::ok_json_message())
                    }
                )*

                let mut router = axum::Router::new()
                    .route("/", axum::routing::get(health_handler));

                $(
                    router = router
                        .nest(
                            concat!("/api/", stringify!($field)),
                            crate::routes::$field::router(),
                        )
                        .nest(
                            concat!("/api/", stringify!($field), "/data"),
                            axum::Router::new()
                                .route("/export",
                                    axum::routing::get([<export_ $field _data>]))
                                .route("/import",
                                    axum::routing::post([<import_ $field _data>]))
                                .route("/clear",
                                    axum::routing::delete([<clear_ $field _data>])),
                        );
                )*
                router.with_state(state)
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════
// Single source of truth for all 70 tools
// ═══════════════════════════════════════════════════════════════

make_app_tools!(
    ac: i_rs_ac::models::AcStore => "ac",
    allergy: i_rs_allergy::models::AllergyStore => "allergy",
    appliance: i_rs_appliance::models::ApplianceStore => "appliance",
    aqua: i_rs_aqua::models::AquaStore => "aqua",
    article: i_rs_article::models::ArticleStore => "article",
    bed: i_rs_bed::models::BedStore => "bed",
    bestby: i_rs_bestby::models::BestByStore => "bestby",
    birthday: i_rs_birthday::models::BirthdayStore => "birthday",
    bookmark: i_rs_bookmark::models::BookmarkStore => "bookmark",
    budget: i_rs_budget::models::BudgetStore => "budget",
    cal: i_rs_cal::models::CalStore => "cal",
    car: i_rs_car::models::CarStore => "car",
    contact: i_rs_contact::models::ContactStore => "contact",
    cycle: i_rs_cycle::models::CycleStore => "cycle",
    cycling: i_rs_cycling::models::CyclingStore => "cycling",
    debt: i_rs_debt::models::DebtStore => "debt",
    deploy: i_rs_deploy::models::DeployStore => "deploy",
    domain: i_rs_domain::models::DomainStore => "domain",
    dose: i_rs_dose::models::DoseStore => "dose",
    event: i_rs_event::models::EventStore => "event",
    exercise: i_rs_exercise::models::ExerciseStore => "exercise",
    fast: i_rs_fast::models::FastStore => "fast",
    feedpet: i_rs_feedpet::models::FeedpetStore => "feedpet",
    filter: i_rs_filter::models::FilterStore => "filter",
    gift: i_rs_gift::models::GiftStore => "gift",
    goal: i_rs_goal::models::GoalStore => "goal",
    grocery: i_rs_grocery::models::GroceryStore => "grocery",
    habit: i_rs_habit::models::HabitStore => "habit",
    height: i_rs_height::models::HeightStore => "height",
    invest: i_rs_invest::models::InvestmentStore => "invest",
    invoice: i_rs_invoice::models::InvoiceStore => "invoice",
    keys: i_rs_keys::models::KeyStore => "keys",
    kv: i_rs_kv::models::KvStore => "kv",
    ledger: i_rs_ledger::models::LedgerStore => "ledger",
    meal: i_rs_meal::models::MealStore => "meal",
    mood: i_rs_mood::models::MoodStore => "mood",
    movie: i_rs_movie::models::MovieStore => "movie",
    note: i_rs_note::models::NoteStore => "note",
    password: i_rs_password::models::PasswordStore => "password",
    petbath: i_rs_petbath::models::PetbathStore => "petbath",
    pig: i_rs_pig::models::PigStore => "pig",
    plant: i_rs_plant::models::PlantStore => "plant",
    podcast: i_rs_podcast::models::PodcastStore => "podcast",
    project: i_rs_project::models::ProjectStore => "project",
    purify: i_rs_purify::models::PurifyStore => "purify",
    quote: i_rs_quote::models::QuoteStore => "quote",
    read: i_rs_read::models::ReadStore => "read",
    recur: i_rs_recur::models::RecurStore => "recur",
    remind: i_rs_remind::models::RemindStore => "remind",
    run: i_rs_run::models::RunStore => "run",
    server: i_rs_server::models::ServerStore => "server",
    sheet: i_rs_sheet::models::SheetStore => "sheet",
    sit: i_rs_sit::models::SitStore => "sit",
    sleep: i_rs_sleep::models::SleepStore => "sleep",
    snippet: i_rs_snippet::models::SnippetStore => "snippet",
    spark: i_rs_spark::models::SparkStore => "spark",
    step: i_rs_step::models::StepStore => "step",
    sub: i_rs_sub::models::SubStore => "sub",
    tax: i_rs_tax::models::TaxStore => "tax",
    tick: i_rs_tick::models::TickStore => "tick",
    time: i_rs_time::models::TimeStore => "time",
    todo: i_rs_todo::models::TodoStore => "todo",
    toothbrush: i_rs_toothbrush::models::ToothbrushStore => "toothbrush",
    towel: i_rs_towel::models::TowelStore => "towel",
    vision: i_rs_vision::models::VisionStore => "vision",
    vocab: i_rs_vocab::models::VocabStore => "vocab",
    walkdog: i_rs_walkdog::models::WalkdogStore => "walkdog",
    want: i_rs_want::models::WantStore => "want",
    water: i_rs_water::models::WaterStore => "water",
    weight: i_rs_weight::models::WeightStore => "weights",
);

// ═══════════════════════════════════════════════════════════════
// Server entry point
// ═══════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    let state = load_state();
    let app = build_base_router(state).layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("i-rs-api server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address. Is port 8080 already in use?");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };
    use tower::ServiceExt;

    fn test_state() -> std::sync::Arc<AppState> {
        let tmp = std::env::temp_dir().join(format!("i-rs-api-test-{}", std::process::id()));
        // SAFETY: test-only, single-threaded at startup
        unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }
        load_state()
    }

    fn test_app() -> Router {
        let state = test_state();
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE]);
        build_base_router(state).layer(cors)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = test_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "i-rs-api");
    }

    #[tokio::test]
    async fn test_ac_crud() {
        let app = test_app();

        // GET /api/ac — empty list
        let res = app
            .clone()
            .oneshot(Request::builder().uri("/api/ac").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // POST /api/ac — create entry
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ac")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"location": "living_room", "tags": ["test"], "remark": ["test"]}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
        assert!(body["data"]["location"] == "living_room");
    }

    #[tokio::test]
    async fn test_add_budget() {
        let app = test_app();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/budget")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "category": "food",
                            "amount": 500.0,
                            "period": "monthly"
                        }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["category"], "food");
    }

    #[tokio::test]
    async fn test_add_weight() {
        let app = test_app();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/weight")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"weight": 75.5}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
    }

    #[tokio::test]
    async fn test_mood_crud() {
        let app = test_app();

        // POST /api/mood
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/mood")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({"mood": "happy", "note": "feeling great"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = test_app();
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/ac/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_bad_request() {
        let app = test_app();
        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/ac")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"invalid": "data"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_data_export() {
        let app = test_app();
        let res = app
            .oneshot(Request::builder().uri("/api/ac/data/export").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap(),
        )
        .unwrap();
        assert_eq!(body["success"], true);
    }

    #[tokio::test]
    async fn test_data_clear() {
        let app = test_app();
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/ac/data/clear")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
