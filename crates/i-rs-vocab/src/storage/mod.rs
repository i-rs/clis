use crate::models::{VocabStore, VocabWord};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<VocabStore> {
    let mut storage = Storage::<VocabStore>::new("vocab");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &VocabStore) -> anyhow::Result<()> {
    let storage = Storage::<VocabStore>::new("vocab");
    storage.save_data(store)
}


pub fn get_word<'a>(store: &'a VocabStore, word_key: &str) -> Option<&'a VocabWord> {
    store.get_word(word_key)
}
