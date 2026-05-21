#![allow(dead_code)]

use crate::utils::atomic_write;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ── Plugin Manifest ──

/// Top-level plugin manifest structure.
/// Parsed from `~/.i-rs-claw/plugins/<name>/plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMeta,
    pub transport: PluginTransport,
}

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Plugin name (must match the directory name).
    pub name: String,
    /// Plugin version string.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Plugin author (optional).
    #[serde(default)]
    pub author: Option<String>,
    /// Plugin homepage URL (optional).
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Transport configuration for a plugin.
/// Mirrors `McpServerConfig` fields for easy conversion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTransport {
    /// Transport type: "stdio" or "sse".
    pub transport_type: String,
    /// Command to execute (for stdio transport).
    #[serde(default)]
    pub command: Option<String>,
    /// Command arguments (for stdio transport).
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// URL endpoint (for sse transport).
    #[serde(default)]
    pub url: Option<String>,
    /// Environment variables in KEY=VAL format (for stdio transport).
    #[serde(default)]
    pub env: Option<Vec<String>>,
}

// ── Plugin State ──

/// Persisted state for all plugins (enabled/disabled).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginState {
    /// Map of plugin name -> enabled state.
    /// Missing entries default to enabled.
    #[serde(default)]
    plugins: HashMap<String, bool>,
}

impl PluginState {
    fn load(path: &PathBuf) -> Self {
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(path)
                && let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
        Self {
            plugins: HashMap::new(),
        }
    }

    fn save(&self, path: &Path) {
        if let Ok(content) = serde_json::to_string_pretty(self)
            && let Err(e) = atomic_write(path, &content) { tracing::error!("持久化写入失败: {}", e); }
    }

    fn is_enabled(&self, name: &str) -> bool {
        self.plugins.get(name).copied().unwrap_or(true)
    }

    fn set_enabled(&mut self, name: &str, enabled: bool) {
        self.plugins.insert(name.to_string(), enabled);
    }

    fn remove(&mut self, name: &str) {
        self.plugins.remove(name);
    }
}

// ── Plugin Manager ──

/// Manages plugin discovery and lifecycle.
pub struct PluginManager {
    /// Base directory for plugins: ~/.i-rs-claw/plugins/
    plugins_dir: PathBuf,
    /// Discovered plugin manifests.
    pub manifests: Vec<PluginManifest>,
    /// Plugin enabled/disabled state.
    state: PluginState,
    /// Path to the state file.
    state_path: PathBuf,
}

impl PluginManager {
    /// Create a new PluginManager and discover plugins.
    pub fn new() -> Self {
        let plugins_dir = dirs::home_dir()
            .map(|h| h.join(".i-rs-claw").join("plugins"))
            .unwrap_or_else(|| PathBuf::from(".i-rs-claw/plugins"));

        let state_path = plugins_dir.join("state.json");
        let state = PluginState::load(&state_path);
        let manifests = Self::discover_manifests(&plugins_dir);

        Self {
            plugins_dir,
            manifests,
            state,
            state_path,
        }
    }

    /// Scan the plugins directory for plugin.toml manifests.
    fn discover_manifests(plugins_dir: &PathBuf) -> Vec<PluginManifest> {
        let mut manifests = Vec::new();

        if !plugins_dir.exists() {
            return manifests;
        }

        let entries = match std::fs::read_dir(plugins_dir) {
            Ok(e) => e,
            Err(_) => return manifests,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }

            match std::fs::read_to_string(&manifest_path) {
                Ok(content) => {
                    match toml::from_str::<PluginManifest>(&content) {
                        Ok(manifest) => {
                            // Validate plugin name matches directory name
                            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                                if manifest.plugin.name == dir_name {
                                    manifests.push(manifest);
                                } else {
                                    tracing::warn!(
                                        "插件目录名 '{}' 与 manifest 中的名称 '{}' 不匹配",
                                        dir_name, manifest.plugin.name
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "解析插件 manifest 失败 '{}': {}",
                                manifest_path.display(),
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("读取插件 manifest 失败 '{}': {}", manifest_path.display(), e);
                }
            }
        }

        manifests
    }

    /// Refresh manifests by re-scanning the plugins directory.
    pub fn refresh(&mut self) {
        self.manifests = Self::discover_manifests(&self.plugins_dir);
        self.state = PluginState::load(&self.state_path);
    }

    /// Get the plugin manifests dir path.
    pub fn plugins_dir(&self) -> &PathBuf {
        &self.plugins_dir
    }

    /// Check if a plugin is enabled.
    pub fn is_enabled(&self, name: &str) -> bool {
        self.state.is_enabled(name)
    }

    /// Enable a plugin.
    pub fn enable(&mut self, name: &str) {
        self.state.set_enabled(name, true);
        self.state.save(&self.state_path);
    }

    /// Disable a plugin.
    pub fn disable(&mut self, name: &str) {
        self.state.set_enabled(name, false);
        self.state.save(&self.state_path);
    }

    /// Remove a plugin's state entry.
    pub fn remove_state(&mut self, name: &str) {
        self.state.remove(name);
        self.state.save(&self.state_path);
    }

    /// Get enabled plugins only.
    pub fn enabled_manifests(&self) -> Vec<&PluginManifest> {
        self.manifests
            .iter()
            .filter(|m| self.is_enabled(&m.plugin.name))
            .collect()
    }

    /// Convert all enabled plugins to McpServerConfig entries.
    /// Plugin-derived configs use a "plugin:" prefix in their name.
    pub fn to_mcp_configs(&self) -> Vec<crate::mcp::McpServerConfig> {
        self.enabled_manifests()
            .iter()
            .map(|manifest| {
                let t = &manifest.transport;
                crate::mcp::McpServerConfig {
                    name: format!("plugin:{}", manifest.plugin.name),
                    transport_type: t.transport_type.clone(),
                    command: t.command.clone(),
                    args: t.args.clone(),
                    url: t.url.clone(),
                    env: t.env.clone(),
                    enabled: true,
                }
            })
            .collect()
    }

    /// Count discovered plugins.
    pub fn plugin_count(&self) -> usize {
        self.manifests.len()
    }

    /// Count enabled plugins.
    pub fn enabled_count(&self) -> usize {
        self.enabled_manifests().len()
    }

    /// Find a manifest by plugin name.
    pub fn find(&self, name: &str) -> Option<&PluginManifest> {
        self.manifests.iter().find(|m| m.plugin.name == name)
    }
}
