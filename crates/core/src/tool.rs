//! Unified trait for all i-rs CLI tools.
//!
//! Every `i-rs-{name}` crate's Store should implement `IrsTool`.
//! The `define_cli_tool!` macro in `macro.rs` uses this trait to generate
//! standard CLI boilerplate (main.rs, CRUD handlers, etc.).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Special capability flags for a tool.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolCapability {
    /// Supports date-range filtering in list (e.g. weight, mood, sleep).
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

/// Pagination parameters for list queries.
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

/// The unified trait every i-rs CLI tool Store must implement.
///
/// ## Associated types
///
/// - `Entity`: the record type that gets stored (e.g. `WeightRecord`).
/// - `Row`: the `#[derive(Tabled)]` type used for table rendering.
/// - `ListItem`: the `#[derive(Serialize)]` type used for JSON list output.
/// - `AddArgs`: clap-parsed struct for the `add` subcommand.
/// - `UpdateArgs`: clap-parsed struct for the `update` subcommand.
///
/// ## Object safety
///
/// The trait is object-safe (no generic methods), which allows
/// `Box<dyn IrsToolEntity>` (entity-level) or `dyn IrsToolRow` (row-level).
/// Full `Box<dyn IrsTool>` is NOT needed in practice — the store type is
/// always known at compile time via monomorphisation in the macro.
pub trait IrsTool: Serialize + serde::de::DeserializeOwned + Default {
    /// Entity type stored in the map.
    type Entity: Serialize + serde::de::DeserializeOwned + Clone;
    /// Table row (`#[derive(Tabled)]`).
    type Row: tabled::Tabled;
    /// JSON list item (`#[derive(Serialize)]`).
    type ListItem: Serialize;

    // ── Metadata ──

    /// Tool name without `i-rs-` prefix (e.g. `"weight"`, `"mood"`).
    fn tool_name() -> &'static str;
    /// One-line description for `--help`.
    fn about() -> &'static str {
        ""
    }
    /// Data filename (defaults to tool_name).
    fn filename() -> &'static str {
        Self::tool_name()
    }
    /// Plural label for counts (e.g. `"records"`, `"entries"`).
    fn label() -> &'static str {
        "entries"
    }
    /// Special capabilities this tool supports.
    fn capabilities() -> Vec<ToolCapability> {
        vec![]
    }

    // ── Store access ──

    /// Immutable reference to the BTreeMap.
    fn entries(&self) -> &BTreeMap<String, Self::Entity>;
    /// Mutable reference to the BTreeMap.
    fn entries_mut(&mut self) -> &mut BTreeMap<String, Self::Entity>;

    // ── CRUD helpers ──

    /// Extract the primary key from an entity.
    fn entity_id(entity: &Self::Entity) -> String;
    /// Generate a new primary key.
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    /// Insert an entity into the store.
    fn add_entry(&mut self, entity: Self::Entity) {
        let id = Self::entity_id(&entity);
        self.entries_mut().insert(id, entity);
    }
    /// Remove an entity by id.
    fn remove_entry(&mut self, id: &str) -> Option<Self::Entity> {
        self.entries_mut().remove(id)
    }
    /// Get an entity by id (immutable).
    fn get_entry(&self, id: &str) -> Option<&Self::Entity> {
        self.entries().get(id)
    }
    /// Get an entity by id (mutable).
    fn get_entry_mut(&mut self, id: &str) -> Option<&mut Self::Entity> {
        self.entries_mut().get_mut(id)
    }

    // ── Row / ListItem conversion ──

    /// Convert an entity to a table row.
    fn to_row(entity: &Self::Entity) -> Self::Row;
    /// Convert an entity to a JSON list item.
    fn to_list_item(entity: &Self::Entity) -> Self::ListItem;

    // ── Entity construction (called by the generated CLI handlers) ──

    /// Create a new entity from the add-subcommand args.
    fn create_entity(
        add_args: &serde_json::Value,
    ) -> anyhow::Result<Self::Entity>;

    /// Mutate an existing entity with update-subcommand args.
    fn update_entity(
        entity: &mut Self::Entity,
        update_args: &serde_json::Value,
    ) -> anyhow::Result<()>;

    // ── Paginated list ──

    /// Paginated query. Default: loads all, filters, slices.
    fn list_paginated(&self, filter: &ListFilter, page: &Pagination) -> Vec<&Self::Entity> {
        let all: Vec<&Self::Entity> = self.entries().values().collect();
        let mut result: Vec<&Self::Entity> = all
            .into_iter()
            .filter(|_e| {
                // Default: no filter — override for tag/keyword filtering
                true
            })
            .collect();
        let total = result.len();
        if page.limit > 0 {
            let start = page.offset.min(total);
            let end = (start + page.limit).min(total);
            result = result[start..end].to_vec();
        }
        result
    }

    // ── Optional stats ──

    /// Compute statistics for the currently loaded entries.
    fn compute_stats(&self) -> Option<serde_json::Value> {
        None
    }
}
