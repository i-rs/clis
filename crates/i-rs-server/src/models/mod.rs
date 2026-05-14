use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(skip)]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ServerStore {
    pub servers: std::collections::BTreeMap<String, Server>,
}


#[derive(Tabled)]
pub struct ServerRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "HOST")]
    host: String,
    #[tabled(rename = "PORT")]
    port: String,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "REMARK")]
    remark: String,
    #[tabled(rename = "CREATED")]
    created_at: String,
    #[tabled(rename = "UPDATED")]
    updated_at: String,
}

impl ServerRow {
    pub fn from_server(server: &Server) -> Self {
        Self {
            name: server.name.clone(),
            host: server.host.clone(),
            port: server.port.to_string(),
            user: server.user.clone().unwrap_or_else(|| "-".to_string()),
            tags: if server.tags.is_empty() {
                "-".to_string()
            } else {
                server.tags.join(", ")
            },
            remark: if server.remark.is_empty() {
                "-".to_string()
            } else {
                server.remark.join(", ")
            },
            created_at: server.created_at.format("%Y-%m-%d %H:%M").to_string(),
            updated_at: server.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}
