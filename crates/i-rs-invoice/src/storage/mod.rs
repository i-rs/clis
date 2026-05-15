use crate::models::{Invoice, InvoiceStore};

i_rs_core::create_store!(InvoiceStore, "invoice");

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
