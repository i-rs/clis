use crate::models::{Contact, ContactStore};
use anyhow::Result;
use chrono::Utc;

use i_rs_core::Storage;

pub fn load_store() -> anyhow::Result<ContactStore> {
    let mut storage = Storage::<ContactStore>::new("contact");
    storage.load()?;
    Ok(storage.data)
}

pub fn save_store(store: &ContactStore) -> anyhow::Result<()> {
    let storage = Storage::<ContactStore>::new("contact");
    storage.save_data(store)
}


pub fn add_contact(
    store: &mut ContactStore,
    name: String,
    phone: String,
    email: String,
    relationship: String,
    tags: Vec<String>,
    remark: Vec<String>,
) -> Result<Contact> {
    let now = Utc::now();
    let contact = Contact {
        name: name.clone(),
        phone,
        email,
        relationship,
        tags,
        remark,
        last_contact: None,
        contact_count: 0,
        created_at: now,
        updated_at: now,
    };
    
    store.add_entry(contact.clone());
    Ok(contact)
}

pub fn delete_contact(store: &mut ContactStore, name: &str) -> Result<Contact> {
    store.remove_entry(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))
}

pub fn update_contact(
    store: &mut ContactStore,
    name: &str,
    phone: Option<String>,
    email: Option<String>,
    relationship: Option<String>,
    tags: Option<Vec<String>>,
    remark: Option<Vec<String>>,
) -> Result<Contact> {
    let contact = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))?;
    
    if let Some(p) = phone {
        contact.phone = p;
    }
    if let Some(e) = email {
        contact.email = e;
    }
    if let Some(r) = relationship {
        contact.relationship = r;
    }
    if let Some(t) = tags {
        contact.tags = t;
    }
    if let Some(rm) = remark {
        contact.remark = rm;
    }
    contact.updated_at = Utc::now();
    
    Ok(contact.clone())
}

#[allow(dead_code)]
pub fn record_contact(store: &mut ContactStore, name: &str) -> Result<Contact> {
    let contact = store.get_entry_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' not found", name))?;
    
    let now = Utc::now();
    contact.last_contact = Some(now);
    contact.contact_count += 1;
    contact.updated_at = now;
    
    Ok(contact.clone())
}
