//! i-rs MCP Server
//!
//! Exposes all 70 i-rs CLI tools as MCP (Model Context Protocol) tools
//! over stdio JSON-RPC 2.0 via rmcp SDK, enabling any MCP-compatible AI client
//! (Claude Desktop, Cursor, Cline, etc.) to read and write personal data.

use std::sync::Arc;

use rmcp::model::{
    CallToolResult, Content, EmptyResult, ErrorData, Implementation, InitializeResult,
    ListToolsResult, RawContent, RawTextContent, ServerCapabilities, ServerResult, Tool,
};
use rmcp::service::{NotificationContext, RequestContext, RoleServer, Service};
use rmcp::ServiceExt;
use serde_json::Value;
use tokio::io::{self as tokio_io};

// ── Thread-safe store ──────────────────────────────────────

/// Thread-safe in-memory store backed by JSON file (same pattern as i-rs-api).
pub struct SharedStore<T> {
    inner: Arc<std::sync::RwLock<T>>,
    filename: String,
}

impl<T: Clone> Clone for SharedStore<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            filename: self.filename.clone(),
        }
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned + Default> SharedStore<T> {
    pub fn load(filename: &str) -> Self {
        let mut storage = i_rs_core::Storage::<T>::new(filename);
        let _ = storage.load();
        let inner = Arc::new(std::sync::RwLock::new(storage.data));
        Self {
            inner,
            filename: filename.to_string(),
        }
    }

    pub fn read<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.inner.read().unwrap_or_else(|e| e.into_inner());
        f(&guard)
    }

    pub fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let result = f(&mut guard);
        let storage = i_rs_core::Storage::<T>::new(&self.filename);
        if let Err(e) = storage.save_data(&guard) {
            eprintln!("[SharedStore] Failed to flush {}: {e}", self.filename);
        }
        result
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Fill auto-generated fields before entity deserialization.
fn fill_defaults(mut args: Value) -> Value {
    let now = chrono::Utc::now();
    if let Some(obj) = args.as_object_mut() {
        obj.entry("id")
            .or_insert_with(|| Value::String(uuid::Uuid::new_v4().to_string()));
        obj.entry("created_at")
            .or_insert_with(|| Value::Number(now.timestamp().into()));
        obj.entry("updated_at")
            .or_insert_with(|| Value::Number(now.timestamp().into()));
    }
    args
}

/// Extract all entries from a store as a JSON array, using runtime serialization
/// to handle any BTreeMap field name.
fn store_values<T: serde::Serialize>(store: &T) -> Result<Vec<Value>, String> {
    let json = serde_json::to_value(store).map_err(|e| format!("serialize: {e}"))?;
    let obj = json.as_object().ok_or("store not an object")?;
    // The store has exactly one field (the BTreeMap) — extract its values
    let field = obj
        .values()
        .next()
        .ok_or("store has no fields")?;
    match field {
        Value::Array(arr) => Ok(arr.clone()),
        Value::Object(map) => Ok(map.values().cloned().collect()),
        _ => Err("unexpected store field type".into()),
    }
}

/// Get a single entry from a store by string key, using runtime serialization.
fn store_get<T: serde::Serialize>(store: &T, key: &str) -> Result<Value, String> {
    let json = serde_json::to_value(store).map_err(|e| format!("serialize: {e}"))?;
    let field = json
        .as_object()
        .ok_or("store not an object")?
        .values()
        .next()
        .ok_or("store has no fields")?;
    match field {
        Value::Object(map) => {
            map.get(key)
                .cloned()
                .ok_or_else(|| format!("entry '{key}' not found"))
        }
        Value::Array(_) => Err("get not supported for array-based stores".into()),
        _ => Err("unexpected store field type".into()),
    }
}

// ── Tool definition ─────────────────────────────────────────

/// Schema for an MCP tool as returned by `tools/list`.
#[derive(Debug, serde::Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

// ── Macro: generate AppState + tools + dispatch ────────────

/// Helper macro for key-type-specific remove logic.
macro_rules! remove_by_key {
    ($store:expr, $map:ident, $key_str:expr, s) => {{
        match $store.$map.remove($key_str) {
            Some(v) => Ok(v),
            None => Err(format!("entry '{}' not found", $key_str)),
        }
    }};
    ($store:expr, $map:ident, $key_str:expr, d) => {{
        let date = chrono::NaiveDate::parse_from_str($key_str, "%Y-%m-%d")
            .map_err(|_| format!("invalid date '{}', expected YYYY-MM-DD", $key_str))?;
        match $store.$map.remove(&date) {
            Some(v) => Ok(v),
            None => Err(format!("entry '{}' not found", $key_str)),
        }
    }};
    ($store:expr, $map:ident, $key_str:expr, u) => {{
        let uuid = uuid::Uuid::parse_str($key_str)
            .map_err(|_| format!("invalid UUID '{}'", $key_str))?;
        match $store.$map.remove(&uuid) {
            Some(v) => Ok(v),
            None => Err(format!("entry '{}' not found", $key_str)),
        }
    }};
}

/// Helper: generic add handler for any entity type.
macro_rules! gen_add_handler {
    ($entity:ty, $args:expr) => {{
        let data = fill_defaults($args);
        let entry: $entity =
            serde_json::from_value(data).map_err(|e| format!("invalid entry data: {e}"))?;
        entry
    }};
}

macro_rules! make_mcp_tools {
    (
        $(
            // (state_field, StoreType, EntityType, map_field, key_type)
            // key_type: s = String, d = NaiveDate, u = Uuid
            ($field:ident, $store:ty, $entity:ty, $map:ident, $kt:tt)
        ),* $(,)?
    ) => {
        paste::paste! {
            pub struct AppState {
                $(pub $field: SharedStore<$store>),*
            }

            fn load_state() -> Arc<AppState> {
                Arc::new(AppState {
                    $($field: SharedStore::<$store>::load(stringify!($field))),*
                })
            }

            /// Return all tool definitions for `tools/list`.
            pub fn get_tool_definitions() -> Vec<ToolDefinition> {
                let mut tools = Vec::new();
                $(
                    let n = stringify!($field);
                    tools.push(ToolDefinition {
                        name: format!("{n}_list"),
                        description: format!("List all {} entries", n),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "tag": {
                                    "type": "string",
                                    "description": "Optional filter by tag"
                                }
                            }
                        }),
                    });
                    tools.push(ToolDefinition {
                        name: format!("{n}_get"),
                        description: format!("Get a {} entry by id", n),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string",
                                    "description": "Entry identifier (date YYYY-MM-DD, name, UUID, or key)"
                                }
                            },
                            "required": ["id"]
                        }),
                    });
                    tools.push(ToolDefinition {
                        name: format!("{n}_add"),
                        description: format!("Add a new {} entry", n),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "description": format!(
                                "Entity data as JSON object for {}. \
                                 Required fields depend on the entity type. \
                                 Auto-generated fields (id, created_at, updated_at) \
                                 are filled if not provided.",
                                n
                            )
                        }),
                    });
                    tools.push(ToolDefinition {
                        name: format!("{n}_delete"),
                        description: format!("Delete a {} entry", n),
                        input_schema: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "id": {
                                    "type": "string",
                                    "description": "Entry identifier (date YYYY-MM-DD, name, UUID, or key)"
                                }
                            },
                            "required": ["id"]
                        }),
                    });
                )*
                tools
            }

            /// Dispatch a tool call to the appropriate handler.
            pub fn handle_tool_call(
                name: &str,
                args: Value,
                state: &AppState,
            ) -> Result<Value, String> {
                $(
                    let n = stringify!($field);
                    if name == format!("{}_list", n) {
                        let entries = state.$field.read(|s| store_values(s))?;
                        let count = entries.len();
                        return Ok(serde_json::json!({
                            "entries": entries,
                            "count": count
                        }));
                    }
                    if name == format!("{}_get", n) {
                        let id = args.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or("missing required 'id' argument")?;
                        let entry = state.$field.read(|s| store_get(s, id))?;
                        return Ok(entry);
                    }
                    if name == format!("{}_add", n) {
                        let entry = gen_add_handler!($entity, args);
                        state.$field.write(|s| {
                            s.add_entry(entry.clone());
                        });
                        return Ok(serde_json::to_value(&entry).map_err(|e| format!("serialize: {e}"))?);
                    }
                    if name == format!("{}_delete", n) {
                        let id = args.get("id")
                            .and_then(|v| v.as_str())
                            .ok_or("missing required 'id' argument")?;
                    state.$field.write(|s| {
                        remove_by_key!(s, $map, id, $kt)
                    })?;
                        return Ok(serde_json::json!({"deleted": true}));
                    }
                )*
                Err(format!("unknown tool: {name}"))
            }
        }
    };
}

// ═══════════════════════════════════════════════════════════════
// Register all 65 BTreeMap-based tools (car/plant/project/goal/budget handled in Phase 2)
// ═══════════════════════════════════════════════════════════════

make_mcp_tools! {
    (ac,          i_rs_ac::models::AcStore,              i_rs_ac::models::AcEntry,              entries, s),
    (allergy,     i_rs_allergy::models::AllergyStore,    i_rs_allergy::models::AllergyEntry,    entries, s),
    (appliance,   i_rs_appliance::models::ApplianceStore, i_rs_appliance::models::Appliance, appliances, s),
    (aqua,        i_rs_aqua::models::AquaStore,          i_rs_aqua::models::AquaEntry,          entries, s),
    (article,     i_rs_article::models::ArticleStore,    i_rs_article::models::Article,          articles, s),
    (bed,         i_rs_bed::models::BedStore,            i_rs_bed::models::BedEntry,            entries, s),
    (bestby,      i_rs_bestby::models::BestByStore,      i_rs_bestby::models::Entity,            entries, s),
    (birthday,    i_rs_birthday::models::BirthdayStore,  i_rs_birthday::models::Birthday,        birthdays, s),
    (bookmark,    i_rs_bookmark::models::BookmarkStore,  i_rs_bookmark::models::Bookmark,        bookmarks, s),
    (cal,         i_rs_cal::models::CalStore,            i_rs_cal::models::CalEntry,            entries, s),
    (contact,     i_rs_contact::models::ContactStore,    i_rs_contact::models::Contact,          entries, s),
    (cycle,       i_rs_cycle::models::CycleStore,        i_rs_cycle::models::CycleEntry,        entries, s),
    (cycling,     i_rs_cycling::models::CyclingStore,    i_rs_cycling::models::CyclingRecord,    records, u),
    (debt,        i_rs_debt::models::DebtStore,          i_rs_debt::models::Debt,                debts, s),
    (deploy,      i_rs_deploy::models::DeployStore,      i_rs_deploy::models::DeployRecord,      entries, s),
    (domain,      i_rs_domain::models::DomainStore,      i_rs_domain::models::Domain,            domains, s),
    (dose,        i_rs_dose::models::DoseStore,          i_rs_dose::models::DoseEntry,          entries, s),
    (event,       i_rs_event::models::EventStore,        i_rs_event::models::Event,             events, s),
    (exercise,    i_rs_exercise::models::ExerciseStore,  i_rs_exercise::models::ExerciseRecord,  records, s),
    (fast,        i_rs_fast::models::FastStore,          i_rs_fast::models::FastEntry,          entries, s),
    (feedpet,     i_rs_feedpet::models::FeedpetStore,    i_rs_feedpet::models::FeedpetEntry,    entries, s),
    (filter,      i_rs_filter::models::FilterStore,      i_rs_filter::models::FilterEntry,      entries, s),
    (gift,        i_rs_gift::models::GiftStore,          i_rs_gift::models::Gift,               gifts, s),
    (grocery,     i_rs_grocery::models::GroceryStore,    i_rs_grocery::models::GroceryItem,      entries, s),
    (habit,       i_rs_habit::models::HabitStore,        i_rs_habit::models::Habit,             entries, s),
    (height,      i_rs_height::models::HeightStore,      i_rs_height::models::HeightRecord,     records, d),
    (invest,      i_rs_invest::models::InvestmentStore,  i_rs_invest::models::Investment,        investments, s),
    (invoice,     i_rs_invoice::models::InvoiceStore,    i_rs_invoice::models::Invoice,          entries, s),
    (keys,        i_rs_keys::models::KeyStore,            i_rs_keys::models::KeyEntry,            entries, s),
    (kv,          i_rs_kv::models::KvStore,              i_rs_kv::models::KvEntry,              entries, s),
    (ledger,      i_rs_ledger::models::LedgerStore,      i_rs_ledger::models::LedgerEntry,      entries, s),
    (meal,        i_rs_meal::models::MealStore,          i_rs_meal::models::MealEntry,          entries, s),
    (mood,        i_rs_mood::models::MoodStore,          i_rs_mood::models::MoodRecord,         records, d),
    (movie,       i_rs_movie::models::MovieStore,        i_rs_movie::models::Movie,             movies, s),
    (note,        i_rs_note::models::NoteStore,          i_rs_note::models::Note,               notes, s),
    (password,    i_rs_password::models::PasswordStore,  i_rs_password::models::PasswordEntry,   entries, s),
    (petbath,     i_rs_petbath::models::PetbathStore,    i_rs_petbath::models::PetbathEntry,    entries, s),
    (pig,         i_rs_pig::models::PigStore,            i_rs_pig::models::PigEntry,            entries, s),
    (podcast,     i_rs_podcast::models::PodcastStore,    i_rs_podcast::models::Podcast,          podcasts, s),
    (purify,      i_rs_purify::models::PurifyStore,      i_rs_purify::models::PurifyEntry,      entries, s),
    (quote,       i_rs_quote::models::QuoteStore,        i_rs_quote::models::Quote,             quotes, s),
    (read,        i_rs_read::models::ReadStore,          i_rs_read::models::Book,               books, s),
    (recur,       i_rs_recur::models::RecurStore,        i_rs_recur::models::RecurEntry,        entries, s),
    (remind,      i_rs_remind::models::RemindStore,      i_rs_remind::models::Remind,            reminds, s),
    (run,         i_rs_run::models::RunStore,            i_rs_run::models::RunRecord,           records, s),
    (server,      i_rs_server::models::ServerStore,      i_rs_server::models::Server,            servers, s),
    (sheet,       i_rs_sheet::models::SheetStore,        i_rs_sheet::models::SheetEntry,        entries, s),
    (sit,         i_rs_sit::models::SitStore,            i_rs_sit::models::SitEntry,            entries, s),
    (sleep,       i_rs_sleep::models::SleepStore,        i_rs_sleep::models::SleepRecord,       entries, s),
    (snippet,     i_rs_snippet::models::SnippetStore,    i_rs_snippet::models::Snippet,          snippets, s),
    (spark,       i_rs_spark::models::SparkStore,        i_rs_spark::models::SparkEntry,        entries, s),
    (step,        i_rs_step::models::StepStore,          i_rs_step::models::StepEntry,          entries, d),
    (sub,         i_rs_sub::models::SubStore,            i_rs_sub::models::SubEntry,            entries, s),
    (tax,         i_rs_tax::models::TaxStore,            i_rs_tax::models::TaxRecord,           entries, s),
    (tick,        i_rs_tick::models::TickStore,          i_rs_tick::models::TickEntry,          entries, s),
    (time,        i_rs_time::models::TimeStore,          i_rs_time::models::TimeEntry,          entries, s),
    (todo,        i_rs_todo::models::TodoStore,          i_rs_todo::models::Todo,               todos, s),
    (toothbrush,  i_rs_toothbrush::models::ToothbrushStore, i_rs_toothbrush::models::ToothbrushEntry, entries, s),
    (towel,       i_rs_towel::models::TowelStore,        i_rs_towel::models::TowelEntry,        entries, s),
    (vision,      i_rs_vision::models::VisionStore,      i_rs_vision::models::VisionRecord,     records, d),
    (vocab,       i_rs_vocab::models::VocabStore,        i_rs_vocab::models::VocabWord,         words, s),
    (walkdog,     i_rs_walkdog::models::WalkdogStore,    i_rs_walkdog::models::WalkdogEntry,    entries, s),
    (want,        i_rs_want::models::WantStore,          i_rs_want::models::WantEntry,          entries, s),
    (water,       i_rs_water::models::WaterStore,        i_rs_water::models::WaterEntry,        entries, s),
    (weight,      i_rs_weight::models::WeightStore,      i_rs_weight::models::WeightRecord,     records, d),
}

// ═══════════════════════════════════════════════════════════════
// rmcp Service implementation
// ═══════════════════════════════════════════════════════════════

/// The MCP server, holding shared application state.
struct McpServer {
    state: Arc<AppState>,
}

impl Service<RoleServer> for McpServer {
    async fn handle_request(
        &self,
        request: rmcp::model::ClientRequest,
        _context: RequestContext<RoleServer>,
    ) -> Result<ServerResult, ErrorData> {
        use rmcp::model::ClientRequest;

        match request {
            ClientRequest::PingRequest(_) => {
                Ok(ServerResult::EmptyResult(EmptyResult {}))
            }
            ClientRequest::InitializeRequest(_) => {
                let capabilities = ServerCapabilities::builder()
                    .enable_tools()
                    .build();
                Ok(ServerResult::InitializeResult(
                    InitializeResult::new(capabilities)
                        .with_server_info(Implementation::new(
                            "i-rs-mcp",
                            env!("CARGO_PKG_VERSION"),
                        )),
                ))
            }
            ClientRequest::ListToolsRequest(_) => {
                let defs = get_tool_definitions();
                let tools: Vec<Tool> = defs
                    .into_iter()
                    .map(|d| {
                        let schema = d.input_schema.as_object().cloned().unwrap_or_default();
                        Tool::new(d.name, d.description, Arc::new(schema))
                    })
                    .collect();
                Ok(ServerResult::ListToolsResult(ListToolsResult::with_all_items(
                    tools,
                )))
            }
            ClientRequest::CallToolRequest(req) => {
                let name = &*req.params.name;
                let args = req.params.arguments.unwrap_or_default();
                match handle_tool_call(name, Value::Object(args), &self.state) {
                    Ok(value) => {
                        Ok(ServerResult::CallToolResult(CallToolResult::structured(value)))
                    }
                    Err(e) => {
                        Ok(ServerResult::CallToolResult(CallToolResult::error(
                            vec![Content {
                                raw: RawContent::Text(RawTextContent {
                                    text: e,
                                    meta: None,
                                }),
                                annotations: None,
                            }],
                        )))
                    }
                }
            }
            other => Err(ErrorData::new(
                rmcp::model::ErrorCode::METHOD_NOT_FOUND,
                format!("method not supported: {other:?}"),
                None,
            )),
        }
    }

    async fn handle_notification(
        &self,
        _notification: rmcp::model::ClientNotification,
        _context: NotificationContext<RoleServer>,
    ) -> Result<(), ErrorData> {
        // Notifications are fire-and-forget; no action needed.
        Ok(())
    }

    fn get_info(&self) -> rmcp::model::ServerInfo {
        InitializeResult::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new("i-rs-mcp", env!("CARGO_PKG_VERSION")))
    }
}

// ═══════════════════════════════════════════════════════════════
// Entry point
// ═══════════════════════════════════════════════════════════════

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = load_state();
    eprintln!(
        "[i-rs-mcp] v{} started ({} stores)",
        env!("CARGO_PKG_VERSION"),
        get_tool_definitions().len() / 4
    );

    let server = McpServer { state };
    let transport = (tokio_io::stdin(), tokio_io::stdout());

    match server.serve(transport).await {
        Ok(running) => {
            running.waiting().await.ok();
        }
        Err(e) => eprintln!("[i-rs-mcp] server error: {e}"),
    }

    eprintln!("[i-rs-mcp] shutdown complete");
    Ok(())
}
