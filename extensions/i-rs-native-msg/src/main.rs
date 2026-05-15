use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvEntry {
    pub value: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KvStore {
    pub entries: BTreeMap<String, KvEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum Request {
    List,
    Get { key: String },
    Set { key: String, value: String },
    Delete { key: String },
    Search { query: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("i-rs")
}

fn get_store_path() -> PathBuf {
    get_config_dir().join("kv.json")
}

fn load_store() -> io::Result<KvStore> {
    let path = get_store_path();
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(KvStore::default())
    }
}

fn save_store(store: &KvStore) -> io::Result<()> {
    let path = get_store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(store)?;
    fs::write(&path, content)
}

fn read_message() -> io::Result<Option<String>> {
    let mut size_buf = [0u8; 4];
    match io::stdin().read_exact(&mut size_buf) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let size = u32::from_le_bytes(size_buf) as usize;
    let mut buf = vec![0u8; size];
    io::stdin().read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn send_message<T: Serialize>(msg: &T) -> io::Result<()> {
    let json = serde_json::to_string(msg).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, e)
    })?;
    let size = json.len() as u32;
    io::stdout().write_all(&size.to_le_bytes())?;
    io::stdout().write_all(json.as_bytes())?;
    io::stdout().flush()
}

fn make_response(success: bool, message: &str, data: Option<serde_json::Value>) -> Response {
    Response {
        success,
        message: Some(message.to_string()),
        data,
    }
}

fn handle_request(req: Request) -> Response {
    match req {
        Request::List => {
            match load_store() {
                Ok(store) => {
                    let data: Vec<_> = store.entries
                        .iter()
                        .map(|(k, v)| serde_json::json!({
                            "key": k,
                            "value": v.value,
                            "updated_at": v.updated_at.timestamp()
                        }))
                        .collect();
                    make_response(true, "OK", Some(serde_json::json!(data)))
                }
                Err(e) => make_response(false, &format!("Failed to load: {}", e), None),
            }
        }
        Request::Get { key } => {
            match load_store() {
                Ok(store) => {
                    match store.entries.get(&key) {
                        Some(entry) => {
                            make_response(true, "OK", Some(serde_json::json!({
                                "key": key,
                                "value": entry.value,
                                "updated_at": entry.updated_at.timestamp()
                            })))
                        }
                        None => make_response(false, "Key not found", None),
                    }
                }
                Err(e) => make_response(false, &format!("Failed to load: {}", e), None),
            }
        }
        Request::Set { key, value } => {
            let entry = KvEntry {
                value,
                updated_at: chrono::Utc::now(),
            };
            match load_store() {
                Ok(mut store) => {
                    store.entries.insert(key.clone(), entry);
                    match save_store(&store) {
                        Ok(_) => make_response(true, "Saved", Some(serde_json::json!({"key": key}))),
                        Err(e) => make_response(false, &format!("Failed to save: {}", e), None),
                    }
                }
                Err(e) => make_response(false, &format!("Failed to load: {}", e), None),
            }
        }
        Request::Delete { key } => {
            match load_store() {
                Ok(mut store) => {
                    match store.entries.remove(&key) {
                        Some(_) => {
                            match save_store(&store) {
                                Ok(_) => make_response(true, "Deleted", None),
                                Err(e) => make_response(false, &format!("Failed to save: {}", e), None),
                            }
                        }
                        None => make_response(false, "Key not found", None),
                    }
                }
                Err(e) => make_response(false, &format!("Failed to load: {}", e), None),
            }
        }
        Request::Search { query } => {
            match load_store() {
                Ok(store) => {
                    let query_lower = query.to_lowercase();
                    let data: Vec<_> = store.entries
                        .iter()
                        .filter(|(k, v)| k.to_lowercase().contains(&query_lower) || v.value.to_lowercase().contains(&query_lower))
                        .map(|(k, v)| serde_json::json!({
                            "key": k,
                            "value": v.value,
                            "updated_at": v.updated_at.timestamp()
                        }))
                        .collect();
                    make_response(true, "OK", Some(serde_json::json!(data)))
                }
                Err(e) => make_response(false, &format!("Failed to load: {}", e), None),
            }
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    loop {
        match read_message()? {
            Some(json) => {
                match serde_json::from_str::<Request>(&json) {
                    Ok(request) => {
                        let response = handle_request(request);
                        if let Err(e) = send_message(&response) {
                            eprintln!("Failed to send response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        let response = make_response(false, &format!("Invalid request: {}", e), None);
                        if let Err(e) = send_message(&response) {
                            eprintln!("Failed to send response: {}", e);
                            break;
                        }
                    }
                }
            }
            None => break,
        }
    }
    Ok(())
}
