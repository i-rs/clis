use crate::models::{VocabStore, VocabWord};


i_rs_core::create_store!(VocabStore, "vocab");


pub fn get_entry<'a>(store: &'a VocabStore, word_key: &str) -> Option<&'a VocabWord> {
    store.get_entry(word_key)
}
