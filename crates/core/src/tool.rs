//! `IrsTool` — the specification trait for every i-rs CLI tool.
//!
//! ## Purpose
//!
//! Every i-rs tool crate's Store type implements this trait.  It serves as:
//!
//! 1. **Contract** for third-party developers building their own i-rs tools.
//! 2. **Discovery** for AI agents (claw, openclaw, qwenpaw) to query tool metadata
//!    and capabilities.
//! 3. **Utilities** — i-rs-core provides generic functions (paginate, etc.) that
//!    work on any `IrsTool`.
//!
//! ## Quick start
//!
//! ```ignore
//! // models/mod.rs
//! impl IrsTool for MyStore {
//!     type Entity = MyRecord;
//!     type Row = MyRow;
//!     type ListItem = MyListItem;
//!
//!     fn tool_name() -> &'static str { "my-tool" }
//!     fn description() -> &'static str { "My custom tool" }
//!
//!     fn entries(&self) -> &BTreeMap<String, MyRecord> { &self.entries }
//!     fn entries_mut(&mut self) -> &mut BTreeMap<String, MyRecord> { &mut self.entries }
//!     fn entity_id(r: &MyRecord) -> String { r.id.clone() }
//!     fn to_row(r: &MyRecord) -> MyRow { MyRow::from_record(r) }
//!     fn to_list_item(r: &MyRecord) -> MyListItem { MyListItem::from(r) }
//! }
//! ```

use serde::Serialize;
use std::collections::BTreeMap;

/// Special capability a tool may advertise to AI agents.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolCapability {
    /// Supports date-range filtering.
    DateRange,
    /// Supports ASCII chart rendering.
    Chart,
    /// Supports calendar view.
    Calendar,
    /// Supports checkin + streak tracking.
    CheckinStreak,
    /// Supports mark-as-done.
    Done,
    /// Supports statistics aggregation.
    Stats,
}

/// Pagination for list queries.
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize, // 0 = no limit
}

/// Filter parameters for list queries.
#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub days: Option<usize>,
    pub tag: Option<String>,
    pub keyword: Option<String>,
}

/// Every i-rs CLI tool Store MUST implement this trait.
pub trait IrsTool: Serialize + serde::de::DeserializeOwned + Default {
    /// The record stored in the entry map.
    type Entity: Serialize + serde::de::DeserializeOwned + Clone;
    /// The `#[derive(Tabled)]` row for table output.
    type Row: tabled::Tabled;
    /// The `#[derive(Serialize)]` item for `--json` list output.
    type ListItem: Serialize;

    // ── Metadata ──

    /// Tool name without the `i-rs-` prefix.
    fn tool_name() -> &'static str;
    /// One-line description (used by claw's tool index).
    fn description() -> &'static str;
    /// Data filename under `~/.i-rs/data/` (defaults to `tool_name`).
    fn filename() -> &'static str {
        Self::tool_name()
    }
    /// Plural label for entry counts (e.g. `"records"`, `"habits"`).
    fn label() -> &'static str {
        "entries"
    }
    /// Special capabilities this tool supports.
    fn capabilities() -> Vec<ToolCapability> {
        vec![]
    }

    // ── Store access ──

    fn entries(&self) -> &BTreeMap<String, Self::Entity>;
    fn entries_mut(&mut self) -> &mut BTreeMap<String, Self::Entity>;

    // ── Key management ──

    /// Extract the primary key from an entity.
    fn entity_id(entity: &Self::Entity) -> String;
    /// Generate a new primary key (default = uuid v4).
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    // ── Display ──

    fn to_row(entity: &Self::Entity) -> Self::Row;
    fn to_list_item(entity: &Self::Entity) -> Self::ListItem;

    // ── Optional stats ──

    fn compute_stats(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Utility: paginate entries from any `IrsTool` store.
pub fn paginate_entries<'a, T: IrsTool>(
    all: &'a [&'a T::Entity],
    page: &Pagination,
) -> (&'a [&'a T::Entity], usize) {
    let total = all.len();
    if page.limit == 0 {
        return (all, total);
    }
    let start = page.offset.min(total);
    let end = (start + page.limit).min(total);
    (&all[start..end], total)
}
