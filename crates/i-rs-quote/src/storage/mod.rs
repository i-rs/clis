use crate::models::{Quote, QuoteStore};


i_rs_core::create_store!(QuoteStore, "quote");


pub fn add_entry(store: &mut QuoteStore, quote: Quote) {
    store.quotes.insert(quote.id.clone(), quote);
}

pub fn remove_entry(store: &mut QuoteStore, id: &str) -> Option<Quote> {
    store.quotes.remove(id)
}

pub fn get_entry<'a>(store: &'a QuoteStore, id: &str) -> Option<&'a Quote> {
    store.quotes.get(id)
}

pub fn get_all_quotes(store: &QuoteStore) -> Vec<&Quote> {
    store.quotes.values().collect()
}

pub fn filter_by_tag<'a>(store: &'a QuoteStore, tag: Option<&str>) -> Vec<&'a Quote> {
    if let Some(tag) = tag {
        store
            .quotes
            .values()
            .filter(|q| q.tags.contains(&tag.to_string()))
            .collect()
    } else {
        store.quotes.values().collect()
    }
}

pub fn filter_by_author<'a>(store: &'a QuoteStore, author: Option<&str>) -> Vec<&'a Quote> {
    if let Some(author) = author {
        let author_lower = author.to_lowercase();
        store
            .quotes
            .values()
            .filter(|q| {
                q.author
                    .as_ref()
                    .is_some_and(|a| a.to_lowercase().contains(&author_lower))
            })
            .collect()
    } else {
        store.quotes.values().collect()
    }
}
