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
    #[allow(dead_code)]
    pub password: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStore {
    pub servers: std::collections::HashMap<String, Server>,
}

impl Default for ServerStore {
    fn default() -> Self {
        Self {
            servers: std::collections::HashMap::new(),
        }
    }
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
        }
    }
}
