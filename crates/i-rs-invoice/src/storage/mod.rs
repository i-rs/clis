use crate::models::{Invoice, InvoiceStore};

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<InvoiceStore> {
    let mut storage = Storage::<InvoiceStore>::new("invoice");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &InvoiceStore) -> anyhow::Result<()> {
    let storage = Storage::<InvoiceStore>::new("invoice");
    storage.save_data(store)
}

pub fn add_entry(store: &mut InvoiceStore, entry: Invoice) {
    store.entries.insert(entry.id.clone(), entry);
}

pub fn remove_entry(store: &mut InvoiceStore, id: &str) -> Option<Invoice> {
    store.entries.remove(id)
}

pub fn get_entry<'a>(store: &'a InvoiceStore, id: &str) -> Option<&'a Invoice> {
    store.entries.get(id)
}

pub fn get_entry_mut<'a>(store: &'a mut InvoiceStore, id: &str) -> Option<&'a mut Invoice> {
    store.entries.get_mut(id)
}

pub fn get_all_entries(store: &InvoiceStore) -> Vec<&Invoice> {
    store.entries.values().collect()
}

pub fn filter_by_reimbursed(store: &InvoiceStore, reimbursed: bool) -> Vec<&Invoice> {
    store
        .entries
        .values()
        .filter(|inv| inv.reimbursed == reimbursed)
        .collect()
}
