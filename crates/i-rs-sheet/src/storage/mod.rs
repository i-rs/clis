use crate::models::{SheetEntry, SheetStore};

i_rs_core::create_store!(SheetStore, "sheet");

pub fn filter_by_tag<'a>(store: &'a SheetStore, tag: Option<&'a str>) -> Vec<&'a SheetEntry> {
    match tag {
        Some(t) => store.entries.values().filter(|e| e.tags.iter().any(|tag| tag == t)).collect(),
        None => store.entries.values().collect(),
    }
}