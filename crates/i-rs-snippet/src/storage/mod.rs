use crate::models::{Snippet, SnippetStore};


i_rs_core::create_store!(SnippetStore, "snippet");


pub fn add_entry(store: &mut SnippetStore, snippet: Snippet) {
    store.snippets.insert(snippet.name.clone(), snippet);
}

pub fn remove_entry(store: &mut SnippetStore, name: &str) -> Option<Snippet> {
    store.snippets.remove(name)
}

pub fn get_entry<'a>(store: &'a SnippetStore, name: &str) -> Option<&'a Snippet> {
    store.snippets.get(name)
}

pub fn get_entry_mut<'a>(store: &'a mut SnippetStore, name: &str) -> Option<&'a mut Snippet> {
    store.snippets.get_mut(name)
}

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
                || s.description.iter().any(|d| d.to_lowercase().contains(&query_lower))
                || s.code.iter().any(|c| c.to_lowercase().contains(&query_lower))
                || s.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .collect()
}
