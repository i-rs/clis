use crate::models::{Quote, QuoteStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<QuoteStore> {
    let mut storage = Storage::<QuoteStore>::new("quote");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &QuoteStore) -> anyhow::Result<()> {
    let storage = Storage::<QuoteStore>::new("quote");
    storage.save_data(store)
}


pub fn add_quote(store: &mut QuoteStore, quote: Quote) {
    store.quotes.insert(quote.id.clone(), quote);
}

pub fn remove_quote(store: &mut QuoteStore, id: &str) -> Option<Quote> {
    store.quotes.remove(id)
}

pub fn get_quote<'a>(store: &'a QuoteStore, id: &str) -> Option<&'a Quote> {
    store.quotes.get(id)
}

pub fn get_all_quotes<'a>(store: &'a QuoteStore) -> Vec<&'a Quote> {
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
                    .map(|a| a.to_lowercase().contains(&author_lower))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        store.quotes.values().collect()
    }
}
