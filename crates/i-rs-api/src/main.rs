mod api;
mod response;
mod routes;
mod store;

use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
};
use tower_http::cors::{CorsLayer, Any};

use store::SharedStore;

pub struct AppState {
    pub ac: SharedStore<i_rs_ac::models::AcStore>,
    pub allergy: SharedStore<i_rs_allergy::models::AllergyStore>,
    pub appliance: SharedStore<i_rs_appliance::models::ApplianceStore>,
    pub aqua: SharedStore<i_rs_aqua::models::AquaStore>,
    pub article: SharedStore<i_rs_article::models::ArticleStore>,
    pub bed: SharedStore<i_rs_bed::models::BedStore>,
    pub bestby: SharedStore<i_rs_bestby::models::BestByStore>,
    pub birthday: SharedStore<i_rs_birthday::models::BirthdayStore>,
    pub bookmark: SharedStore<i_rs_bookmark::models::BookmarkStore>,
    pub budget: SharedStore<i_rs_budget::models::BudgetStore>,
    pub cal: SharedStore<i_rs_cal::models::CalStore>,
    pub car: SharedStore<i_rs_car::models::CarStore>,
    pub contact: SharedStore<i_rs_contact::models::ContactStore>,
    pub cycle: SharedStore<i_rs_cycle::models::CycleStore>,
    pub cycling: SharedStore<i_rs_cycling::models::CyclingStore>,
    pub debt: SharedStore<i_rs_debt::models::DebtStore>,
    pub deploy: SharedStore<i_rs_deploy::models::DeployStore>,
    pub domain: SharedStore<i_rs_domain::models::DomainStore>,
    pub dose: SharedStore<i_rs_dose::models::DoseStore>,
    pub event: SharedStore<i_rs_event::models::EventStore>,
    pub exercise: SharedStore<i_rs_exercise::models::ExerciseStore>,
    pub fast: SharedStore<i_rs_fast::models::FastStore>,
    pub feedpet: SharedStore<i_rs_feedpet::models::FeedpetStore>,
    pub filter: SharedStore<i_rs_filter::models::FilterStore>,
    pub gift: SharedStore<i_rs_gift::models::GiftStore>,
    pub goal: SharedStore<i_rs_goal::models::GoalStore>,
    pub grocery: SharedStore<i_rs_grocery::models::GroceryStore>,
    pub habit: SharedStore<i_rs_habit::models::HabitStore>,
    pub height: SharedStore<i_rs_height::models::HeightStore>,
    pub invest: SharedStore<i_rs_invest::models::InvestmentStore>,
    pub invoice: SharedStore<i_rs_invoice::models::InvoiceStore>,
    pub keys: SharedStore<i_rs_keys::models::KeyStore>,
    pub kv: SharedStore<i_rs_kv::models::KvStore>,
    pub ledger: SharedStore<i_rs_ledger::models::LedgerStore>,
    pub meal: SharedStore<i_rs_meal::models::MealStore>,
    pub mood: SharedStore<i_rs_mood::models::MoodStore>,
    pub movie: SharedStore<i_rs_movie::models::MovieStore>,
    pub note: SharedStore<i_rs_note::models::NoteStore>,
    pub password: SharedStore<i_rs_password::models::PasswordStore>,
    pub petbath: SharedStore<i_rs_petbath::models::PetbathStore>,
    pub pig: SharedStore<i_rs_pig::models::PigStore>,
    pub plant: SharedStore<i_rs_plant::models::PlantStore>,
    pub podcast: SharedStore<i_rs_podcast::models::PodcastStore>,
    pub project: SharedStore<i_rs_project::models::ProjectStore>,
    pub purify: SharedStore<i_rs_purify::models::PurifyStore>,
    pub quote: SharedStore<i_rs_quote::models::QuoteStore>,
    pub read: SharedStore<i_rs_read::models::ReadStore>,
    pub recur: SharedStore<i_rs_recur::models::RecurStore>,
    pub remind: SharedStore<i_rs_remind::models::RemindStore>,
    pub run: SharedStore<i_rs_run::models::RunStore>,
    pub server: SharedStore<i_rs_server::models::ServerStore>,
    pub sheet: SharedStore<i_rs_sheet::models::SheetStore>,
    pub sit: SharedStore<i_rs_sit::models::SitStore>,
    pub sleep: SharedStore<i_rs_sleep::models::SleepStore>,
    pub snippet: SharedStore<i_rs_snippet::models::SnippetStore>,
    pub spark: SharedStore<i_rs_spark::models::SparkStore>,
    pub step: SharedStore<i_rs_step::models::StepStore>,
    pub sub: SharedStore<i_rs_sub::models::SubStore>,
    pub tax: SharedStore<i_rs_tax::models::TaxStore>,
    pub tick: SharedStore<i_rs_tick::models::TickStore>,
    pub time: SharedStore<i_rs_time::models::TimeStore>,
    pub todo: SharedStore<i_rs_todo::models::TodoStore>,
    pub toothbrush: SharedStore<i_rs_toothbrush::models::ToothbrushStore>,
    pub towel: SharedStore<i_rs_towel::models::TowelStore>,
    pub vision: SharedStore<i_rs_vision::models::VisionStore>,
    pub vocab: SharedStore<i_rs_vocab::models::VocabStore>,
    pub walkdog: SharedStore<i_rs_walkdog::models::WalkdogStore>,
    pub want: SharedStore<i_rs_want::models::WantStore>,
    pub water: SharedStore<i_rs_water::models::WaterStore>,
    pub weight: SharedStore<i_rs_weight::models::WeightStore>,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    let state = Arc::new(AppState {
        ac: SharedStore::load("ac"),
        allergy: SharedStore::load("allergy"),
        appliance: SharedStore::load("appliance"),
        aqua: SharedStore::load("aqua"),
        article: SharedStore::load("article"),
        bed: SharedStore::load("bed"),
        bestby: SharedStore::load("bestby"),
        birthday: SharedStore::load("birthday"),
        bookmark: SharedStore::load("bookmark"),
        budget: SharedStore::load("budget"),
        cal: SharedStore::load("cal"),
        car: SharedStore::load("car"),
        contact: SharedStore::load("contact"),
        cycle: SharedStore::load("cycle"),
        cycling: SharedStore::load("cycling"),
        debt: SharedStore::load("debt"),
        deploy: SharedStore::load("deploy"),
        domain: SharedStore::load("domain"),
        dose: SharedStore::load("dose"),
        event: SharedStore::load("event"),
        exercise: SharedStore::load("exercise"),
        fast: SharedStore::load("fast"),
        feedpet: SharedStore::load("feedpet"),
        filter: SharedStore::load("filter"),
        gift: SharedStore::load("gift"),
        goal: SharedStore::load("goal"),
        grocery: SharedStore::load("grocery"),
        habit: SharedStore::load("habit"),
        height: SharedStore::load("height"),
        invest: SharedStore::load("invest"),
        invoice: SharedStore::load("invoice"),
        keys: SharedStore::load("keys"),
        kv: SharedStore::load("kv"),
        ledger: SharedStore::load("ledger"),
        meal: SharedStore::load("meal"),
        mood: SharedStore::load("mood"),
        movie: SharedStore::load("movie"),
        note: SharedStore::load("note"),
        password: SharedStore::load("password"),
        petbath: SharedStore::load("petbath"),
        pig: SharedStore::load("pig"),
        plant: SharedStore::load("plant"),
        podcast: SharedStore::load("podcast"),
        project: SharedStore::load("project"),
        purify: SharedStore::load("purify"),
        quote: SharedStore::load("quote"),
        read: SharedStore::load("read"),
        recur: SharedStore::load("recur"),
        remind: SharedStore::load("remind"),
        run: SharedStore::load("run"),
        server: SharedStore::load("server"),
        sheet: SharedStore::load("sheet"),
        sit: SharedStore::load("sit"),
        sleep: SharedStore::load("sleep"),
        snippet: SharedStore::load("snippet"),
        spark: SharedStore::load("spark"),
        step: SharedStore::load("step"),
        sub: SharedStore::load("sub"),
        tax: SharedStore::load("tax"),
        tick: SharedStore::load("tick"),
        time: SharedStore::load("time"),
        todo: SharedStore::load("todo"),
        toothbrush: SharedStore::load("toothbrush"),
        towel: SharedStore::load("towel"),
        vision: SharedStore::load("vision"),
        vocab: SharedStore::load("vocab"),
        walkdog: SharedStore::load("walkdog"),
        want: SharedStore::load("want"),
        water: SharedStore::load("water"),
        weight: SharedStore::load("weights"),
    });

    let app = Router::new()
        .route("/", axum::routing::get(handlers::health))
        .nest("/api/ac", routes::ac::router())
        .nest("/api/allergy", routes::allergy::router())
        .nest("/api/appliance", routes::appliance::router())
        .nest("/api/aqua", routes::aqua::router())
        .nest("/api/article", routes::article::router())
        .nest("/api/bed", routes::bed::router())
        .nest("/api/bestby", routes::bestby::router())
        .nest("/api/birthday", routes::birthday::router())
        .nest("/api/bookmark", routes::bookmark::router())
        .nest("/api/budget", routes::budget::router())
        .nest("/api/cal", routes::cal::router())
        .nest("/api/car", routes::car::router())
        .nest("/api/contact", routes::contact::router())
        .nest("/api/cycle", routes::cycle::router())
        .nest("/api/cycling", routes::cycling::router())
        .nest("/api/debt", routes::debt::router())
        .nest("/api/deploy", routes::deploy::router())
        .nest("/api/domain", routes::domain::router())
        .nest("/api/dose", routes::dose::router())
        .nest("/api/event", routes::event::router())
        .nest("/api/exercise", routes::exercise::router())
        .nest("/api/fast", routes::fast::router())
        .nest("/api/feedpet", routes::feedpet::router())
        .nest("/api/filter", routes::filter::router())
        .nest("/api/gift", routes::gift::router())
        .nest("/api/goal", routes::goal::router())
        .nest("/api/grocery", routes::grocery::router())
        .nest("/api/habit", routes::habit::router())
        .nest("/api/height", routes::height::router())
        .nest("/api/invest", routes::invest::router())
        .nest("/api/invoice", routes::invoice::router())
        .nest("/api/keys", routes::keys::router())
        .nest("/api/kv", routes::kv::router())
        .nest("/api/ledger", routes::ledger::router())
        .nest("/api/meal", routes::meal::router())
        .nest("/api/mood", routes::mood::router())
        .nest("/api/movie", routes::movie::router())
        .nest("/api/note", routes::note::router())
        .nest("/api/password", routes::password::router())
        .nest("/api/petbath", routes::petbath::router())
        .nest("/api/pig", routes::pig::router())
        .nest("/api/plant", routes::plant::router())
        .nest("/api/podcast", routes::podcast::router())
        .nest("/api/project", routes::project::router())
        .nest("/api/purify", routes::purify::router())
        .nest("/api/quote", routes::quote::router())
        .nest("/api/read", routes::read::router())
        .nest("/api/recur", routes::recur::router())
        .nest("/api/remind", routes::remind::router())
        .nest("/api/run", routes::run::router())
        .nest("/api/server", routes::server::router())
        .nest("/api/sheet", routes::sheet::router())
        .nest("/api/sit", routes::sit::router())
        .nest("/api/sleep", routes::sleep::router())
        .nest("/api/snippet", routes::snippet::router())
        .nest("/api/spark", routes::spark::router())
        .nest("/api/step", routes::step::router())
        .nest("/api/sub", routes::sub::router())
        .nest("/api/tax", routes::tax::router())
        .nest("/api/tick", routes::tick::router())
        .nest("/api/time", routes::time::router())
        .nest("/api/todo", routes::todo::router())
        .nest("/api/toothbrush", routes::toothbrush::router())
        .nest("/api/towel", routes::towel::router())
        .nest("/api/vision", routes::vision::router())
        .nest("/api/vocab", routes::vocab::router())
        .nest("/api/walkdog", routes::walkdog::router())
        .nest("/api/want", routes::want::router())
        .nest("/api/water", routes::water::router())
        .nest("/api/weight", routes::weight::router())
        .with_state(state)
        .layer(cors);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("i-rs-api server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address. Is port 8080 already in use?");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

mod handlers {
    use axum::Json;

    pub async fn health() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "success": true,
            "status": "ok",
            "service": "i-rs-api",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "REST API for i-rs CLI tools",
            "endpoints": [
                "/api/ac",
                "/api/allergy",
                "/api/appliance",
                "/api/aqua",
                "/api/article",
                "/api/bed",
                "/api/bestby",
                "/api/birthday",
                "/api/bookmark",
                "/api/budget",
                "/api/cal",
                "/api/car",
                "/api/contact",
                "/api/cycle",
                "/api/cycling",
                "/api/debt",
                "/api/deploy",
                "/api/domain",
                "/api/dose",
                "/api/event",
                "/api/exercise",
                "/api/fast",
                "/api/feedpet",
                "/api/filter",
                "/api/gift",
                "/api/goal",
                "/api/grocery",
                "/api/habit",
                "/api/height",
                "/api/invest",
                "/api/invoice",
                "/api/keys",
                "/api/kv",
                "/api/ledger",
                "/api/meal",
                "/api/mood",
                "/api/movie",
                "/api/note",
                "/api/password",
                "/api/petbath",
                "/api/pig",
                "/api/plant",
                "/api/podcast",
                "/api/project",
                "/api/purify",
                "/api/quote",
                "/api/read",
                "/api/recur",
                "/api/remind",
                "/api/run",
                "/api/server",
                "/api/sheet",
                "/api/sit",
                "/api/sleep",
                "/api/snippet",
                "/api/spark",
                "/api/step",
                "/api/sub",
                "/api/tax",
                "/api/tick",
                "/api/time",
                "/api/todo",
                "/api/toothbrush",
                "/api/towel",
                "/api/vision",
                "/api/vocab",
                "/api/walkdog",
                "/api/want",
                "/api/water",
                "/api/weight",
            ]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn test_state() -> Arc<AppState> {
        // Use a temp config dir so tests don't touch real data
        let tmp = std::env::temp_dir().join(format!("i-rs-api-test-{}", std::process::id()));
        // SAFETY: test-only, single-threaded at startup
        unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }

        Arc::new(AppState {
            ac: SharedStore::load("ac"),
            allergy: SharedStore::load("allergy"),
            appliance: SharedStore::load("appliance"),
            aqua: SharedStore::load("aqua"),
            article: SharedStore::load("article"),
            bed: SharedStore::load("bed"),
            bestby: SharedStore::load("bestby"),
            birthday: SharedStore::load("birthday"),
            bookmark: SharedStore::load("bookmark"),
            budget: SharedStore::load("budget"),
            cal: SharedStore::load("cal"),
            car: SharedStore::load("car"),
            contact: SharedStore::load("contact"),
            cycle: SharedStore::load("cycle"),
            cycling: SharedStore::load("cycling"),
            debt: SharedStore::load("debt"),
            deploy: SharedStore::load("deploy"),
            domain: SharedStore::load("domain"),
            dose: SharedStore::load("dose"),
            event: SharedStore::load("event"),
            exercise: SharedStore::load("exercise"),
            fast: SharedStore::load("fast"),
            feedpet: SharedStore::load("feedpet"),
            filter: SharedStore::load("filter"),
            gift: SharedStore::load("gift"),
            goal: SharedStore::load("goal"),
            grocery: SharedStore::load("grocery"),
            habit: SharedStore::load("habit"),
            height: SharedStore::load("height"),
            invest: SharedStore::load("invest"),
            invoice: SharedStore::load("invoice"),
            keys: SharedStore::load("keys"),
            kv: SharedStore::load("kv"),
            ledger: SharedStore::load("ledger"),
            meal: SharedStore::load("meal"),
            mood: SharedStore::load("mood"),
            movie: SharedStore::load("movie"),
            note: SharedStore::load("note"),
            password: SharedStore::load("password"),
            petbath: SharedStore::load("petbath"),
            pig: SharedStore::load("pig"),
            plant: SharedStore::load("plant"),
            podcast: SharedStore::load("podcast"),
            project: SharedStore::load("project"),
            purify: SharedStore::load("purify"),
            quote: SharedStore::load("quote"),
            read: SharedStore::load("read"),
            recur: SharedStore::load("recur"),
            remind: SharedStore::load("remind"),
            run: SharedStore::load("run"),
            server: SharedStore::load("server"),
            sheet: SharedStore::load("sheet"),
            sit: SharedStore::load("sit"),
            sleep: SharedStore::load("sleep"),
            snippet: SharedStore::load("snippet"),
            spark: SharedStore::load("spark"),
            step: SharedStore::load("step"),
            sub: SharedStore::load("sub"),
            tax: SharedStore::load("tax"),
            tick: SharedStore::load("tick"),
            time: SharedStore::load("time"),
            todo: SharedStore::load("todo"),
            toothbrush: SharedStore::load("toothbrush"),
            towel: SharedStore::load("towel"),
            vision: SharedStore::load("vision"),
            vocab: SharedStore::load("vocab"),
            walkdog: SharedStore::load("walkdog"),
            want: SharedStore::load("want"),
            water: SharedStore::load("water"),
            weight: SharedStore::load("weights"),
        })
    }

    fn test_app() -> Router {
        let state = test_state();
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([header::CONTENT_TYPE]);

        Router::new()
            .route("/", axum::routing::get(handlers::health))
            .nest("/api/ac", routes::ac::router())
            .nest("/api/allergy", routes::allergy::router())
            .nest("/api/appliance", routes::appliance::router())
            .nest("/api/aqua", routes::aqua::router())
            .nest("/api/article", routes::article::router())
            .nest("/api/bed", routes::bed::router())
            .nest("/api/bestby", routes::bestby::router())
            .nest("/api/birthday", routes::birthday::router())
            .nest("/api/bookmark", routes::bookmark::router())
            .nest("/api/budget", routes::budget::router())
            .nest("/api/cal", routes::cal::router())
            .nest("/api/car", routes::car::router())
            .nest("/api/contact", routes::contact::router())
            .nest("/api/cycle", routes::cycle::router())
            .nest("/api/cycling", routes::cycling::router())
            .nest("/api/debt", routes::debt::router())
            .nest("/api/deploy", routes::deploy::router())
            .nest("/api/domain", routes::domain::router())
            .nest("/api/dose", routes::dose::router())
            .nest("/api/event", routes::event::router())
            .nest("/api/exercise", routes::exercise::router())
            .nest("/api/fast", routes::fast::router())
            .nest("/api/feedpet", routes::feedpet::router())
            .nest("/api/filter", routes::filter::router())
            .nest("/api/gift", routes::gift::router())
            .nest("/api/goal", routes::goal::router())
            .nest("/api/grocery", routes::grocery::router())
            .nest("/api/habit", routes::habit::router())
            .nest("/api/height", routes::height::router())
            .nest("/api/invest", routes::invest::router())
            .nest("/api/invoice", routes::invoice::router())
            .nest("/api/keys", routes::keys::router())
            .nest("/api/kv", routes::kv::router())
            .nest("/api/ledger", routes::ledger::router())
            .nest("/api/meal", routes::meal::router())
            .nest("/api/mood", routes::mood::router())
            .nest("/api/movie", routes::movie::router())
            .nest("/api/note", routes::note::router())
            .nest("/api/password", routes::password::router())
            .nest("/api/petbath", routes::petbath::router())
            .nest("/api/pig", routes::pig::router())
            .nest("/api/plant", routes::plant::router())
            .nest("/api/podcast", routes::podcast::router())
            .nest("/api/project", routes::project::router())
            .nest("/api/purify", routes::purify::router())
            .nest("/api/quote", routes::quote::router())
            .nest("/api/read", routes::read::router())
            .nest("/api/recur", routes::recur::router())
            .nest("/api/remind", routes::remind::router())
            .nest("/api/run", routes::run::router())
            .nest("/api/server", routes::server::router())
            .nest("/api/sheet", routes::sheet::router())
            .nest("/api/sit", routes::sit::router())
            .nest("/api/sleep", routes::sleep::router())
            .nest("/api/snippet", routes::snippet::router())
            .nest("/api/spark", routes::spark::router())
            .nest("/api/step", routes::step::router())
            .nest("/api/sub", routes::sub::router())
            .nest("/api/tax", routes::tax::router())
            .nest("/api/tick", routes::tick::router())
            .nest("/api/time", routes::time::router())
            .nest("/api/todo", routes::todo::router())
            .nest("/api/toothbrush", routes::toothbrush::router())
            .nest("/api/towel", routes::towel::router())
            .nest("/api/vision", routes::vision::router())
            .nest("/api/vocab", routes::vocab::router())
            .nest("/api/walkdog", routes::walkdog::router())
            .nest("/api/want", routes::want::router())
            .nest("/api/water", routes::water::router())
            .nest("/api/weight", routes::weight::router())
            .with_state(state)
            .layer(cors)
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
}
