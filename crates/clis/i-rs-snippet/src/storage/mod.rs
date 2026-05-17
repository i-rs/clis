use crate::models::{Snippet, SnippetStore};

i_rs_core::create_store!(SnippetStore, "snippet");

pub fn filter_by_tag<'a>(store: &'a SnippetStore, tag: Option<&str>) -> Vec<&'a Snippet> {
    if let Some(tag) = tag {
        store
            .snippets
            .values()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.snippets.values().collect()
    }
}

pub fn search_snippets<'a>(store: &'a SnippetStore, query: &str) -> Vec<&'a Snippet> {
    let query_lower = query.to_lowercase();
    store
        .snippets
        .values()
        .filter(|s| {
            s.name.to_lowercase().contains(&query_lower)
                || s.language.to_lowercase().contains(&query_lower)
                || s.description
                    .iter()
                    .any(|d| d.to_lowercase().contains(&query_lower))
                || s.code
                    .iter()
                    .any(|c| c.to_lowercase().contains(&query_lower))
                || s.tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect()
}
