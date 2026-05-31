//! `IrsTool` — the specification trait for every i-rs CLI tool.
//!
//! ## Role
//!
//! | Layer | What | Example |
//! |-------|------|---------|
//! | **CLI protocol** (runtime) | Subprocess: `i-rs-{tool} {cmd} --json` | claw calls via shell |
//! | **SKILL.md** (runtime) | Natural language docs for AI agents | `i-rs-{tool} skill teach` |
//! | **`IrsTool` trait** (compile-time) | Type-level contract for Rust developers | `impl IrsTool for MyStore` |
//!
//! Each layer is independent but they describe the same tool from different angles.
//! The trait mirrors the CLI protocol in Rust types but does NOT replace it —
//! a tool must still be callable as a standalone binary.
//!
//! ## For third-party developers
//!
//! Implement `IrsTool` on your Store type to make your tool discoverable by
//! `claw` and other AI agents via compile-time linking.  Your tool remains a
//! normal CLI binary usable by anyone — the trait just adds type-level metadata.
//!
//! Required: 7 one-liners + 3 type aliases. Everything else has sensible defaults.
//!
//! ```ignore
//! impl IrsTool for MyStore {
//!     type Entity = MyRecord;
//!     type Row = MyRow;
//!     type ListItem = MyListItem;
//!
//!     fn tool_name() -> &'static str { "my-tool" }
//!     fn description() -> &'static str { "Description for claw's tool index" }
//!
//!     fn entries(&self) -> &BTreeMap<String, MyRecord> { &self.entries }
//!     fn entries_mut(&mut self) -> &mut BTreeMap<String, MyRecord> { &mut self.entries }
//!     fn entity_id(r: &MyRecord) -> String { r.id.clone() }
//!     fn to_row(r: &MyRecord) -> MyRow { MyRow::from_record(r) }
//!     fn to_list_item(r: &MyRecord) -> MyListItem { MyListItem::from(r) }
//!
//!     // Optional: advertise special capabilities
//!     fn capabilities() -> Vec<ToolCapability> { vec![ToolCapability::Stats] }
//!     fn custom_commands() -> &'static [&'static str] { &["checkin"] }
//! }
//! ```

use serde::Serialize;
use std::collections::BTreeMap;

// ═══════════════════════════════════════════════════════════════════
//  Supporting types
// ═══════════════════════════════════════════════════════════════════

/// Capability flags that claw uses for tool routing and context assembly.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolCapability {
    /// List command supports `--days` date-range filtering.
    DateRange,
    /// Supports ASCII chart output.
    Chart,
    /// Supports calendar view output.
    Calendar,
    /// Supports checkin + streak computation.
    CheckinStreak,
    /// Supports `done` / mark-as-complete.
    Done,
    /// Supports `stats` computation.
    Stats,
}

/// Pagination for `list` queries.
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize, // 0 = no limit
}

// ═══════════════════════════════════════════════════════════════════
//  IrsTool trait
// ═══════════════════════════════════════════════════════════════════

/// The specification trait for every i-rs CLI tool Store.
///
/// ## Required methods (7)
///
/// | Method | What to return |
/// |--------|---------------|
/// | `tool_name()` | `"weight"`, `"mood"`, ... |
/// | `description()` | One-line summary for claw's tool index |
/// | `entries()` | `&self.entries` |
/// | `entries_mut()` | `&mut self.entries` |
/// | `entity_id(r)` | `r.id.clone()` (or `r.key.clone()`) |
/// | `to_row(r)` | `MyRow::from_record(r)` |
/// | `to_list_item(r)` | `MyListItem::from(r)` |
///
/// ## Optional overrides (all have sensible defaults)
///
/// | Method | Default | Override when... |
/// |--------|---------|-----------------|
/// | `filename()` | = `tool_name()` | File name differs from tool name |
/// | `label()` | `"entries"` | Use `"records"`, `"habits"`, etc. |
/// | `capabilities()` | `[]` | Tool has DateRange/Chart/Stats etc. |
/// | `custom_commands()` | `&[]` | Tool has non-standard commands (checkin, search, ...) |
/// | `generate_id()` | `uuid::Uuid::new_v4()` | Custom ID scheme |
/// | `compute_stats()` | `None` | Tool computes aggregate stats |
pub trait IrsTool: Serialize + serde::de::DeserializeOwned + Default {
    /// The record type stored in the entry map.
    type Entity: Serialize + serde::de::DeserializeOwned + Clone;
    /// The `#[derive(Tabled)]` row for table display.
    type Row: tabled::Tabled;
    /// The `#[derive(Serialize)]` item for `--json` list output.
    type ListItem: Serialize;

    // ── Identity ──

    /// Tool name without the `i-rs-` prefix.
    fn tool_name() -> &'static str;
    /// CLI binary name.
    fn binary_name() -> &'static str {
        // concat! is not const-evaluable in trait default methods, so
        // we provide it as a convenience but it requires alloc.
        // Third parties can override if they use a different naming scheme.
        "unknown"
    }
    /// One-line description used by claw's tool index.
    fn description() -> &'static str;

    // ── Storage hints ──

    /// Data filename under `~/.i-rs/data/` (defaults to `tool_name`).
    fn filename() -> &'static str {
        Self::tool_name()
    }
    /// Plural label for entry counts (e.g. `"records"`, `"habits"`).
    fn label() -> &'static str {
        "entries"
    }

    // ── Commands ──

    /// Special capabilities — used by claw for routing decisions.
    /// Standard CRUD (add/delete/get/list/update) is implicit.
    fn capabilities() -> Vec<ToolCapability> {
        vec![]
    }
    /// Non-standard command names beyond add/delete/get/list/update.
    ///
    /// Used by claw to discover tool-specific commands without calling
    /// `skill teach` first.  Example: `&["checkin"]` for habit,
    /// `&["search", "stats", "copy", "rename"]` for kv.
    fn custom_commands() -> &'static [&'static str] {
        &[]
    }

    // ── Data access ──

    fn entries(&self) -> &BTreeMap<String, Self::Entity>;
    fn entries_mut(&mut self) -> &mut BTreeMap<String, Self::Entity>;

    // ── Key management ──

    /// Extract the primary key from an entity.
    fn entity_id(entity: &Self::Entity) -> String;
    /// Generate a new unique primary key.
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    // ── Display conversion ──

    fn to_row(entity: &Self::Entity) -> Self::Row;
    fn to_list_item(entity: &Self::Entity) -> Self::ListItem;

    // ── Aggregate ──

    /// Compute aggregate statistics for the current data.
    fn compute_stats(&self) -> Option<serde_json::Value> {
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
//  Generic utilities (work on any IrsTool)
// ═══════════════════════════════════════════════════════════════════

/// Paginate entries from any `IrsTool` store.
///
/// ```ignore
/// let all = store.entries().values().collect::<Vec<_>>();
/// let (page, total) = paginate_entries::<WeightStore>(&all, &pagination);
/// ```
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
