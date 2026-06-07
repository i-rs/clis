use i_rs_claw_core::config::Config;
use std::io::{self, Write};

// =============================================
// Interactive Config Wizard
// =============================================

pub fn run_config() -> anyhow::Result<()> {
    let config_path = claw_dir().join("config.toml");
    let mut cfg = if config_path.exists() {
        Config::load().unwrap_or_else(|_| Config::new())
    } else {
        println!("未发现配置文件，开始交互式设置...\n");
        Config::new()
    };

    // ── Provider Selection ──
    let provider_names: Vec<&str> = i_rs_claw_core::providers::ProviderKind::all()
        .iter()
        .map(|p| p.as_str())
        .collect();
    let provider_default = cfg.provider.as_str();
    print!(
        "Provider [{}] ({}): ",
        provider_default,
        provider_names.join("/")
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.provider = trimmed.parse().expect("invalid provider");
    }

    // ── API Key ──
    let current = if cfg.api_key.is_empty() {
        String::new()
    } else {
        format!(
            " [{}...{}]",
            &cfg.api_key[..4.min(cfg.api_key.len())],
            &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
        )
    };
    print!("API Key{}: ", current);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.api_key = trimmed;
    }

    // ── Base URL ──
    print!("Base URL [{}]: ", cfg.base_url);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.base_url = trimmed;
    }

    // ── Model ──
    print!("Model [{}]: ", cfg.model);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.model = trimmed;
    }

    // ── Search API Key (optional) ──
    let search_current = cfg
        .search_api_key
        .as_ref()
        .map(|k| {
            if k.is_empty() {
                String::new()
            } else {
                format!(
                    " [{}...{}]",
                    &k[..4.min(k.len())],
                    &k[k.len().saturating_sub(4)..]
                )
            }
        })
        .unwrap_or_default();
    print!("Search API Key{} (留空使用 DuckDuckGo): ", search_current);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.search_api_key = Some(trimmed);
    }

    // ── Search Base URL (optional) ──
    let search_url_default = cfg
        .search_base_url
        .as_deref()
        .unwrap_or("DuckDuckGo (free)");
    print!("Search Base URL [{}]: ", search_url_default);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.search_base_url = Some(trimmed);
    }

    // ── MCP Servers (optional) ──
    let mcp_count = cfg.mcp_servers.len();
    println!("\n  MCP 服务器 (当前 {} 个):", mcp_count);
    println!("  MCP (Model Context Protocol) 允许连接外部工具服务器。");
    println!("  配置格式: name|command|arg1 arg2|KEY=VAL");
    println!("  例如: filesystem|npx|-y @modelcontextprotocol/server-filesystem /path");
    println!("  留空则跳过 MCP 配置，可后续在配置文件中修改。");
    print!("添加 MCP 服务器 (留空跳过): ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let mcp_input = input.trim().to_string();
    if !mcp_input.is_empty() {
        if let Some(server) = parse_mcp_server(&mcp_input) {
            cfg.mcp_servers.push(server);
            println!("  ✓ 已添加 MCP 服务器");
        } else {
            println!("  ⚠ 格式无效，期望: name|command|arg1 arg2|KEY=VAL");
        }
    }

    // ── Timezone ──
    let current_tz = i_rs_claw_core::utils::tz_label(cfg.tz_offset);
    println!("\n  Timezone (时区)");
    println!("  当前: {}", current_tz);
    println!("  格式: +08:00 / -05:00 / UTC / UTC+8 / 8 (留空=系统本地)");
    print!("Timezone [{}]: ", current_tz);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        cfg.timezone = Some(trimmed);
        cfg.tz_offset = i_rs_claw_core::utils::parse_timezone(cfg.timezone.as_deref());
    }

    // ── Storage Backend ──
    println!("\n  Storage (存储后端)");
    println!("  选项: file / sqlite / mysql / postgres / mongodb / redis");
    println!("  file 为 JSON 文件存储（默认），sqlite 需要编译 --features sqlite");
    let storage_default = match cfg.storage.backend {
        i_rs_claw_core::storage::StorageBackend::File => "file",
        i_rs_claw_core::storage::StorageBackend::Sqlite => "sqlite",
        i_rs_claw_core::storage::StorageBackend::Mysql => "mysql",
        i_rs_claw_core::storage::StorageBackend::Postgres => "postgres",
        i_rs_claw_core::storage::StorageBackend::Mongo => "mongodb",
        i_rs_claw_core::storage::StorageBackend::Redis => "redis",
    };
    print!("Storage 后端 [{}]: ", storage_default);
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    if !trimmed.is_empty() {
        match trimmed.as_str() {
            "file" => cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::File,
            "sqlite" => {
                cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::Sqlite;
                let sqlite_default = cfg
                    .storage
                    .sqlite_path
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .unwrap_or("~/.i-rs/claw/claw.db");
                print!("  SQLite 路径 [{}]: ", sqlite_default);
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let sp = input.trim().to_string();
                if !sp.is_empty() {
                    cfg.storage.sqlite_path = Some(std::path::PathBuf::from(
                        sp.replace('~', &dirs::home_dir().unwrap().to_string_lossy()),
                    ));
                } else {
                    cfg.storage.sqlite_path = Some(std::path::PathBuf::from(
                        sqlite_default.replace('~', &dirs::home_dir().unwrap().to_string_lossy()),
                    ));
                }
            }
            "mysql" => {
                cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::Mysql;
                let url_default = cfg
                    .storage
                    .sql_url
                    .as_deref()
                    .unwrap_or("mysql://localhost:3306/i_rs_claw");
                print!("  MySQL URL [{}]: ", url_default);
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let url = input.trim().to_string();
                cfg.storage.sql_url = if url.is_empty() {
                    Some(url_default.to_string())
                } else {
                    Some(url)
                };
            }
            "postgres" => {
                cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::Postgres;
                let url_default = cfg
                    .storage
                    .sql_url
                    .as_deref()
                    .unwrap_or("postgres://localhost:5432/i_rs_claw");
                print!("  PostgreSQL URL [{}]: ", url_default);
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let url = input.trim().to_string();
                cfg.storage.sql_url = if url.is_empty() {
                    Some(url_default.to_string())
                } else {
                    Some(url)
                };
            }
            "mongodb" => {
                cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::Mongo;
                let url_default = cfg
                    .storage
                    .mongo_url
                    .as_deref()
                    .unwrap_or("mongodb://localhost:27017");
                print!("  MongoDB URL [{}]: ", url_default);
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let url = input.trim().to_string();
                cfg.storage.mongo_url = if url.is_empty() {
                    Some(url_default.to_string())
                } else {
                    Some(url)
                };
                print!("  MongoDB 数据库 [i_rs_claw]: ");
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let db = input.trim().to_string();
                cfg.storage.mongo_database = if db.is_empty() {
                    Some("i_rs_claw".to_string())
                } else {
                    Some(db)
                };
            }
            "redis" => {
                cfg.storage.backend = i_rs_claw_core::storage::StorageBackend::Redis;
                let url_default = cfg
                    .storage
                    .redis_url
                    .as_deref()
                    .unwrap_or("redis://localhost:6379/0");
                print!("  Redis URL [{}]: ", url_default);
                io::stdout().flush()?;
                input.clear();
                io::stdin().read_line(&mut input)?;
                let url = input.trim().to_string();
                cfg.storage.redis_url = if url.is_empty() {
                    Some(url_default.to_string())
                } else {
                    Some(url)
                };
            }
            _ => println!("  ⚠ 未知后端 '{}'，保留原值", trimmed),
        }
    }

    // ── Save ──
    let needs_api_key = cfg.provider != i_rs_claw_core::providers::ProviderKind::Ollama;
    if needs_api_key && cfg.api_key.is_empty() {
        anyhow::bail!("{} 需要 API Key，配置未保存", cfg.provider);
    }

    cfg.save()?;
    let total_tools = cfg.i_rs_tools.len();
    let enabled_count = if cfg.enabled_tools.is_empty() {
        total_tools
    } else {
        cfg.enabled_tools.len()
    };

    let tz_display = cfg.timezone.as_deref().unwrap_or("系统本地");
    let storage_label = match cfg.storage.backend {
        i_rs_claw_core::storage::StorageBackend::File => "file",
        i_rs_claw_core::storage::StorageBackend::Sqlite => "sqlite",
        i_rs_claw_core::storage::StorageBackend::Mysql => "mysql",
        i_rs_claw_core::storage::StorageBackend::Postgres => "postgres",
        i_rs_claw_core::storage::StorageBackend::Mongo => "mongodb",
        i_rs_claw_core::storage::StorageBackend::Redis => "redis",
    };

    println!("\n配置摘要：");
    println!("  Provider: {}", cfg.provider);
    println!(
        "  API Key: {}...{}",
        &cfg.api_key[..4.min(cfg.api_key.len())],
        &cfg.api_key[cfg.api_key.len().saturating_sub(4)..]
    );
    println!("  Base URL: {}", cfg.base_url);
    println!("  Model: {}", cfg.model);
    println!(
        "  搜索: {}",
        cfg.search_base_url
            .as_deref()
            .unwrap_or("DuckDuckGo (free)")
    );
    println!("  MCP 服务器: {} 个", cfg.mcp_servers.len());
    println!("  时区: {}", tz_display);
    println!("  存储后端: {}", storage_label);
    println!(
        "  工具: {} ({} 个 / 总 {} 个)",
        if cfg.enabled_tools.is_empty() {
            "全部启用"
        } else {
            "部分启用"
        },
        enabled_count,
        total_tools,
    );
    println!("  运行 `i-rs-claw tools` 管理工具开关");

    Ok(())
}

pub(crate) fn claw_dir() -> std::path::PathBuf {
    i_rs_claw_core::utils::claw_dir().expect("无法获取用户主目录")
}

/// Parse MCP server config from user input.
/// Format: name|command|arg1 arg2|KEY=VAL
fn parse_mcp_server(input: &str) -> Option<i_rs_claw_core::mcp::McpServerConfig> {
    let parts: Vec<&str> = input.splitn(4, '|').collect();
    if parts.len() < 2 {
        return None;
    }
    let name = parts[0].trim().to_string();
    let command = parts[1].trim().to_string();
    if name.is_empty() || command.is_empty() {
        return None;
    }
    let args = parts
        .get(2)
        .map(|s| s.split_whitespace().map(|a| a.to_string()).collect());
    let env = parts
        .get(3)
        .map(|s| s.split_whitespace().map(|e| e.to_string()).collect());
    Some(i_rs_claw_core::mcp::McpServerConfig {
        name,
        transport_type: "stdio".to_string(),
        command: Some(command),
        args,
        url: None,
        env,
        enabled: true,
    })
}
