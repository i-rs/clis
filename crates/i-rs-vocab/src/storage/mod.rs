use crate::models::{VocabStore, VocabWord};


i_rs_core::create_store!(VocabStore, "vocab");


pub fn get_word<'a>(store: &'a VocabStore, word_key: &str) -> Option<&'a VocabWord> {
    store.get_word(word_key)
}
