# API Key 安全存储 — 设计方案

> 状态: Draft  
> 日期: 2026-05-20  
> 范围: i-rs-claw  
> 优先级: P0（安全底线）

---

## 1. 背景

### 1.1 现状

当前 API Key 以明文存储在 `~/.i-rs-claw/config.toml` 中：

```toml
provider = "openai"
api_key = "sk-xxx..."

[agents.analyst]
provider = "anthropic"
api_key = "sk-ant-xxx..."
```

**风险**:
- 同机其他用户可通过 `cat ~/.i-rs-claw/config.toml` 读取
- 文件权限默认 0644（同用户组可读）
- 备份到云盘时 Key 明文泄露
- Git 误提交风险（虽有 `.gitignore`，但配置本身不安全）

### 1.2 多 Provider / 多 Agent 场景

i-rs-claw 支持多 Agent 架构，每个 Agent 可独立配置 Provider 和 API Key：

```toml
# 顶层默认配置
provider = "openai"
api_key = "sk-openai-xxx"

[agents.analyst]
provider = "anthropic"
api_key = "sk-ant-xxx"

[agents.coder]
provider = "openai"
api_key = "sk-openai-yyy"  # 可能用不同的 OpenAI Key
```

**设计必须支持**：
- 每个 Provider 一个默认 Key（如 `openai` 的默认 Key）
- 每个 Agent 可覆盖自己的 Key
- 加载时按 Agent → Provider 链式解析

### 1.3 目标

- API Key 不在配置文件中以明文存储
- 优先使用操作系统原生密钥存储（Keychain / Secret Service）
- 支持多 Provider、多 Agent 的 Key 独立存储
- 无桌面环境时回退到文件 + 0600 权限
- 环境变量始终作为最高优先级覆盖（已实现）
- 用户体验：首次设置交互式输入，后续无感使用

---

## 2. 架构设计

### 2.1 Keyring 存储结构

Keyring 使用 `service + account` 二元组标识每条密钥。i-rs-claw 的存储结构：

```
Keyring Service: "i-rs-claw"
├── Account: "provider:openai:default"      → sk-openai-xxx
├── Account: "provider:anthropic:default"   → sk-ant-xxx
├── Account: "agent:analyst:api-key"        → sk-ant-analyst-xxx
├── Account: "agent:coder:api-key"          → sk-openai-coder-yyy
└── Account: "agent:researcher:api-key"     → sk-ollama-researcher
```

**命名规则**：
- **Provider 默认 Key**: `provider:{provider_type}:default`
- **Agent 覆盖 Key**: `agent:{agent_id}:api-key`

### 2.2 加载优先级链（5 级）

```
┌──────────────────────────────────────────────────────────┐
│            resolve_api_key_for(agent_id)                  │
├──────────────────────────────────────────────────────────┤
│  1. 环境变量 I_RS_CLAW_API_KEY_{AGENT_ID_UPPER}         │
│     → 如 I_RS_CLAW_API_KEY_ANALYST                       │
│     → 兼容旧变量 I_RS_CLAW_API_KEY                        │
├──────────────────────────────────────────────────────────┤
│  2. Agent 专属 Keyring                                   │
│     → account: "agent:{agent_id}:api-key"                │
│     → 适用于 Agent 使用不同 Key 的场景                    │
├──────────────────────────────────────────────────────────┤
│  3. Provider 默认 Keyring                                │
│     → account: "provider:{provider_type}:default"        │
│     → 适用于同一 Provider 共用 Key 的场景                 │
├──────────────────────────────────────────────────────────┤
│  4. Agent 配置文件中的明文 api_key                        │
│     → [agents.{agent_id}].api_key                        │
│     → 兼容旧配置 / 无 Keyring 环境                        │
├──────────────────────────────────────────────────────────┤
│  5. 顶层配置文件中的明文 api_key                          │
│     → 顶层 api_key 字段                                  │
│     → 最终回退                                           │
└──────────────────────────────────────────────────────────┘
```

### 2.3 存储策略

| 场景 | 存储位置 | 用户操作 |
|------|---------|---------|
| 有桌面环境（macOS/Linux GUI） | OS Keyring（按 Provider/Agent 分条） | `i-rs-claw config` 交互式输入 |
| 无桌面环境（SSH/服务器） | 文件 + 0600 | `i-rs-claw config` 或手动编辑 |
| CI/CD / 容器 | 环境变量 | 设置 `I_RS_CLAW_API_KEY` 或 `I_RS_CLAW_API_KEY_ANALYST` |

---

## 3. 数据流

### 3.1 首次设置

```
用户运行: i-rs-claw config

1. 选择存储类型:
   ❯ Provider 默认 Key
     Agent 专属 Key

2a. 选 Provider:
    ❯ openai
      anthropic
      ollama (不需要 Key)
    输入 API Key: ********

2b. 选 Agent:
    ❯ default
      analyst
      coder
    输入 API Key: ********

3. 尝试写入 OS Keyring
   ├── 成功 → 配置文件中不存储对应 api_key 字段
   └── 失败 → 回退到文件存储 + 设置 0600 权限
4. 提示 "✓ API Key 已安全存储"
```

### 3.2 运行时加载

```
AppCore 初始化 → 为每个 Agent 调用 resolve_api_key_for(agent_id)

resolve_api_key_for("analyst"):
  1. env: I_RS_CLAW_API_KEY_ANALYST → 未设置
  2. env: I_RS_CLAW_API_KEY → 未设置
  3. keyring: "agent:analyst:api-key" → 未找到
  4. keyring: "provider:anthropic:default" → 找到! 返回 sk-ant-xxx
  （停止查找，不再继续第 5 级）
```

### 3.3 更新 Key

```
用户运行: i-rs-claw config --update-key

1. 选择类型: Provider 默认 / Agent 专属
2. 选择具体 Provider 或 Agent
3. 提示输入新 API Key（隐藏输入）
4. 写入 OS Keyring（覆盖旧值）
5. 如果之前是文件回退，同时更新文件 + 权限
```

### 3.4 删除 Key

```
用户运行: i-rs-claw config --delete-key

1. 选择类型: Provider 默认 / Agent 专属
2. 选择具体 Provider 或 Agent
3. 从 OS Keyring 删除对应条目
4. 从配置文件中移除对应 api_key 字段
```

### 3.5 列出已存储的 Key

```
用户运行: i-rs-claw config --list-keys

已存储的 API Key:
  Provider:
    openai     ✓ (Keychain)
    anthropic  ✓ (Keychain)
  Agent:
    analyst    ✓ (Keychain, 覆盖 anthropic)
    coder      - (未设置, 使用 openai 默认)
```

---

## 4. 模块设计

### 4.1 新增/修改文件

```
crates/claw/src/
├── config.rs          — 修改（新增 resolve_api_key_for 方法）
├── keyring.rs         — 新增（OS Keyring 封装，支持多 Provider/Agent）
└── cli.rs             — 修改（config 子命令增强）
```

### 4.2 keyring.rs — OS Keyring 封装

```rust
//! OS-native secure credential storage wrapper.
//!
//! Supports multiple API keys organized by provider and agent:
//! - Provider default keys: `provider:{type}:default`
//! - Agent override keys: `agent:{id}:api-key`
//!
//! Platform backends:
//! - macOS: Keychain
//! - Linux: Secret Service (D-Bus / libsecret)
//! - Windows: Credential Manager

use keyring::Entry;

const KEYRING_SERVICE: &str = "i-rs-claw";

// ── Keyring Account Naming ──

/// Build the keyring account name for a provider's default key.
fn provider_account(provider: &str) -> String {
    format!("provider:{}:default", provider)
}

/// Build the keyring account name for an agent's override key.
fn agent_account(agent_id: &str) -> String {
    format!("agent:{}:api-key", agent_id)
}

// ── Provider Keys ──

/// Store an API key for a provider's default account.
pub fn store_provider_key(provider: &str, key: &str) -> Result<(), KeyringError> {
    let account = provider_account(provider);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.set_password(key)
        .map_err(|e| KeyringError::Store(account, e.to_string()))?;
    Ok(())
}

/// Retrieve the stored API key for a provider's default account.
pub fn get_provider_key(provider: &str) -> Result<String, KeyringError> {
    let account = provider_account(provider);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.get_password()
        .map_err(|e| KeyringError::Retrieve(account, e.to_string()))
}

/// Delete the stored API key for a provider's default account.
pub fn delete_provider_key(provider: &str) -> Result<(), KeyringError> {
    let account = provider_account(provider);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.delete_credential()
        .map_err(|e| KeyringError::Delete(account, e.to_string()))?;
    Ok(())
}

// ── Agent Keys ──

/// Store an API key for a specific agent (overrides provider default).
pub fn store_agent_key(agent_id: &str, key: &str) -> Result<(), KeyringError> {
    let account = agent_account(agent_id);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.set_password(key)
        .map_err(|e| KeyringError::Store(account, e.to_string()))?;
    Ok(())
}

/// Retrieve the stored API key for a specific agent.
pub fn get_agent_key(agent_id: &str) -> Result<String, KeyringError> {
    let account = agent_account(agent_id);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.get_password()
        .map_err(|e| KeyringError::Retrieve(account, e.to_string()))
}

/// Delete the stored API key for a specific agent.
pub fn delete_agent_key(agent_id: &str) -> Result<(), KeyringError> {
    let account = agent_account(agent_id);
    let entry = Entry::new(KEYRING_SERVICE, &account)
        .map_err(|e| KeyringError::CreateEntry(account, e.to_string()))?;
    entry.delete_credential()
        .map_err(|e| KeyringError::Delete(account, e.to_string()))?;
    Ok(())
}

// ── Utility ──

/// Check if the OS keyring is accessible (quick probe).
pub fn is_keyring_available() -> bool {
    // Try creating a test entry and deleting it
    let account = "__probe__";
    match Entry::new(KEYRING_SERVICE, account) {
        Ok(entry) => {
            entry.set_password("probe").is_ok()
                && entry.delete_credential().is_ok()
        }
        Err(_) => false,
    }
}

/// List all stored keyring accounts (for --list-keys).
pub fn list_stored_keys() -> Vec<(String, String)> {
    // Returns vec of (type, name) pairs
    // e.g. ("provider", "openai"), ("agent", "analyst")
    // Note: keyring crate doesn't provide enumeration,
    // so we probe known patterns or maintain a local index.
    // For now, return empty — can be enhanced later.
    Vec::new()
}

// ── Error Types ──

/// Keyring operation errors.
#[derive(Debug)]
pub enum KeyringError {
    CreateEntry(String, String),  // (account, message)
    Store(String, String),
    Retrieve(String, String),
    Delete(String, String),
}

impl std::fmt::Display for KeyringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyringError::CreateEntry(account, msg) =>
                write!(f, "创建密钥环条目失败 [{}]: {}", account, msg),
            KeyringError::Store(account, msg) =>
                write!(f, "存储密钥失败 [{}]: {}", account, msg),
            KeyringError::Retrieve(account, msg) =>
                write!(f, "读取密钥失败 [{}]: {}", account, msg),
            KeyringError::Delete(account, msg) =>
                write!(f, "删除密钥失败 [{}]: {}", account, msg),
        }
    }
}
```

### 4.3 config.rs — 修改

#### 4.3.1 Config 结构体变更

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub provider: String,
    /// API key — optional when stored in OS keyring.
    /// If present, used as fallback after keyring lookup fails.
    #[serde(default)]
    pub api_key: Option<String>,  // ← 改为 Option
    // ... 其余字段不变
}
```

#### 4.3.2 AgentConfig 结构体变更

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    #[serde(default)]
    pub provider: Option<String>,
    /// Agent-specific API key — optional when stored in OS keyring.
    #[serde(default)]
    pub api_key: Option<String>,  // ← 改为 Option
    // ... 其余字段不变
}
```

#### 4.3.3 新增 resolve_api_key_for 方法

```rust
impl Config {
    /// Resolve the API key for a specific agent using the 5-level priority chain:
    /// 1. Env var I_RS_CLAW_API_KEY_{AGENT_ID_UPPER}
    /// 2. Env var I_RS_CLAW_API_KEY (fallback)
    /// 3. Agent-specific keyring entry
    /// 4. Provider default keyring entry
    /// 5. Agent config file api_key
    /// 6. Top-level config file api_key
    pub fn resolve_api_key_for(&self, agent_id: &str) -> anyhow::Result<String> {
        let resolved = self.agent_config(agent_id);
        
        // Priority 1: Agent-specific environment variable
        let env_var = format!("I_RS_CLAW_API_KEY_{}", agent_id.to_uppercase());
        if let Ok(key) = std::env::var(&env_var) {
            if !key.is_empty() {
                return Ok(key);
            }
        }
        
        // Priority 2: Generic environment variable (backward compatible)
        if let Ok(key) = std::env::var("I_RS_CLAW_API_KEY") {
            if !key.is_empty() {
                return Ok(key);
            }
        }
        
        // Priority 3: Agent-specific keyring
        if let Ok(key) = crate::keyring::get_agent_key(agent_id) {
            if !key.is_empty() {
                return Ok(key);
            }
        }
        
        // Priority 4: Provider default keyring
        if let Ok(key) = crate::keyring::get_provider_key(&resolved.provider) {
            if !key.is_empty() {
                return Ok(key);
            }
        }
        
        // Priority 5: Agent config file (backward compatible)
        if let Some(ref key) = resolved.api_key {
            if !key.is_empty() {
                return Ok(key.clone());
            }
        }
        
        // Priority 6: Top-level config file (final fallback)
        if let Some(ref key) = self.api_key {
            if !key.is_empty() {
                return Ok(key.clone());
            }
        }
        
        anyhow::bail!(
            "未找到 API Key (agent='{}', provider='{}')。\n\
             请通过以下方式之一设置：\n\
             1. 运行 `i-rs-claw config --update-key` 交互式设置\n\
             2. 设置环境变量 I_RS_CLAW_API_KEY 或 I_RS_CLAW_API_KEY_{}\n\
             3. 在配置文件中设置 api_key 字段",
            agent_id, resolved.provider, agent_id.to_uppercase()
        )
    }
}
```

#### 4.3.4 Config::load 变更

```rust
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // ... 现有读取逻辑 ...
        
        let config: Config = toml::from_str(&content)?;
        
        // 移除原有的 api_key 空值检查
        // 验证延迟到 resolve_api_key_for 调用时
        
        // ... 其余逻辑不变 ...
        Ok(config)
    }
}
```

#### 4.3.5 Config::save 变更

```rust
impl Config {
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        
        // 创建一个用于序列化的副本，排除已存入 keyring 的 Key
        let mut config_for_save = self.clone();
        
        // 如果 keyring 可用，顶层 api_key 不需要明文存储
        if crate::keyring::is_keyring_available() {
            config_for_save.api_key = None;
        }
        
        let content = toml::to_string_pretty(&config_for_save)?;
        atomic_write(&config_path, &content)?;
        
        // 设置文件权限为 0600（仅所有者可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&config_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&config_path, perms)?;
        }
        
        println!("✓ 配置已保存: {}", config_path.display());
        Ok(())
    }
}
```

### 4.4 cli.rs — config 子命令增强

```rust
// config 子命令增强
"config" => {
    let args: Vec<String> = std::env::args().collect();
    let update_key = args.iter().any(|a| a == "--update-key");
    let delete_key = args.iter().any(|a| a == "--delete-key");
    let list_keys = args.iter().any(|a| a == "--list-keys");
    
    if list_keys {
        return cmd_list_keys();
    }
    
    if update_key {
        return cmd_update_key();
    }
    
    if delete_key {
        return cmd_delete_key();
    }
    
    // 原有 config 创建/显示逻辑
    // ...
}

/// Interactive command: store a new API key.
fn cmd_update_key() -> anyhow::Result<()> {
    println!("选择存储类型:");
    println!("  1. Provider 默认 Key（所有使用该 Provider 的 Agent 共用）");
    println!("  2. Agent 专属 Key（覆盖 Provider 默认 Key）");
    print!("请选择 [1/2]: ");
    std::io::stdout().flush()?;
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    
    match choice.trim() {
        "1" => cmd_update_provider_key(),
        "2" => cmd_update_agent_key(),
        _ => {
            eprintln!("无效选择");
            Ok(())
        }
    }
}

fn cmd_update_provider_key() -> anyhow::Result<()> {
    println!("\n支持的 Provider:");
    println!("  1. openai");
    println!("  2. anthropic");
    println!("  3. ollama (不需要 API Key)");
    print!("请选择 [1/2/3]: ");
    std::io::stdout().flush()?;
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    
    let provider = match choice.trim() {
        "1" => "openai",
        "2" => "anthropic",
        "3" => {
            println!("Ollama 不需要 API Key");
            return Ok(());
        }
        _ => {
            eprintln!("无效选择");
            return Ok(());
        }
    };
    
    print!("\n输入 {} API Key: ", provider);
    std::io::stdout().flush()?;
    let key = read_hidden_input()?;
    
    if crate::keyring::store_provider_key(provider, &key).is_ok() {
        println!("✓ {} API Key 已安全存储到系统密钥环", provider);
    } else {
        println!("⚠ 系统密钥环不可用，将存储到配置文件");
        let mut config = Config::load().unwrap_or_else(|_| Config::new());
        if provider == config.provider {
            config.api_key = Some(key);
        }
        config.save()?;
    }
    Ok(())
}

fn cmd_update_agent_key() -> anyhow::Result<()> {
    let config = Config::load()?;
    let agent_ids = config.agent_ids();
    
    println!("\n可用的 Agent:");
    for (i, id) in agent_ids.iter().enumerate() {
        println!("  {}. {}", i + 1, id);
    }
    print!("请选择: ");
    std::io::stdout().flush()?;
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    
    let idx: usize = match choice.trim().parse() {
        Ok(n) if n > 0 && n <= agent_ids.len() => n - 1,
        _ => {
            eprintln!("无效选择");
            return Ok(());
        }
    };
    
    let agent_id = &agent_ids[idx];
    print!("\n输入 Agent '{}' 的 API Key: ", agent_id);
    std::io::stdout().flush()?;
    let key = read_hidden_input()?;
    
    if crate::keyring::store_agent_key(agent_id, &key).is_ok() {
        println!("✓ Agent '{}' API Key 已安全存储到系统密钥环", agent_id);
    } else {
        println!("⚠ 系统密钥环不可用，将存储到配置文件");
        // Fallback: store in config file
    }
    Ok(())
}

fn cmd_delete_key() -> anyhow::Result<()> {
    // 类似 update_key 的交互流程，调用 delete_provider_key / delete_agent_key
    // ...
    Ok(())
}

fn cmd_list_keys() -> anyhow::Result<()> {
    let config = Config::load()?;
    
    println!("已配置的 API Key 状态:\n");
    
    // Provider keys
    println!("Provider 默认 Key:");
    for provider in &["openai", "anthropic", "ollama"] {
        let status = match crate::keyring::get_provider_key(provider) {
            Ok(_) => "✓ (系统密钥环)",
            Err(_) => "-",
        };
        println!("  {:<12} {}", provider, status);
    }
    
    // Agent keys
    println!("\nAgent 专属 Key:");
    for agent_id in config.agent_ids() {
        let resolved = config.agent_config(&agent_id);
        let status = match crate::keyring::get_agent_key(&agent_id) {
            Ok(_) => format!("✓ (系统密钥环, 覆盖 {})", resolved.provider),
            Err(_) => format!("- (使用 {} 默认)", resolved.provider),
        };
        println!("  {:<12} {}", agent_id, status);
    }
    
    Ok(())
}

/// Read input from stdin without echoing (for password/key input).
fn read_hidden_input() -> std::io::Result<String> {
    // 优先使用 rpassword crate
    rpassword::read_password()
}
```

---

## 5. 依赖变更

### 5.1 Cargo.toml

```toml
[dependencies]
# 新增
keyring = "3"
rpassword = "7"
```

### 5.2 Linux 特殊处理

Linux 服务器通常没有 D-Bus / Secret Service 守护进程。

**策略**:
1. `keyring` crate 在 Linux 上默认尝试 Secret Service
2. 如果检测到无桌面环境（`$DBUS_SESSION_BUS_ADDRESS` 不存在），跳过 keyring，直接回退文件
3. 最终回退到文件 + 0600 权限

```rust
// keyring.rs 中自动检测
fn is_desktop_linux() -> bool {
    std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok()
        || std::env::var("XDG_RUNTIME_DIR").is_ok()
}
```

---

## 6. 迁移方案

### 6.1 自动迁移（首次启动时）

```rust
/// 首次启动时，尝试将配置文件中的明文 Key 迁移到 Keyring。
fn migrate_api_keys_to_keyring(config: &Config) {
    if !crate::keyring::is_keyring_available() {
        return;
    }
    
    // 迁移顶层默认 Key
    if let Some(ref key) = config.api_key {
        if !key.is_empty() {
            if crate::keyring::get_provider_key(&config.provider).is_err() {
                let _ = crate::keyring::store_provider_key(&config.provider, key);
                tracing::info!("顶层 API Key 已自动迁移到系统密钥环 [provider:{}:default]", config.provider);
            }
        }
    }
    
    // 迁移各 Agent 的 Key
    for agent_id in config.agent_ids() {
        let resolved = config.agent_config(&agent_id);
        if let Some(ref key) = resolved.api_key {
            if !key.is_empty() {
                if crate::keyring::get_agent_key(&agent_id).is_err() {
                    let _ = crate::keyring::store_agent_key(&agent_id, key);
                    tracing::info!("Agent '{}' API Key 已自动迁移到系统密钥环", agent_id);
                }
            }
        }
    }
}
```

### 6.2 用户提示

```
⚠ 检测到配置文件中存在明文 API Key。
  已自动迁移到系统密钥环。
  下次保存配置时，配置文件中的明文 Key 将被移除。
```

---

## 7. 边界情况处理

| 场景 | 处理方式 |
|------|---------|
| Keyring 服务被其他应用占用 | 使用唯一 service name `i-rs-claw` |
| Linux 无 D-Bus | 跳过 keyring，直接回退文件 + 0600 |
| Keyring 存储失败 | 回退到文件 + 0600 + warn 日志 |
| 配置文件权限不是 0600 | 每次 save 时自动修正为 0600 |
| 用户同时设置环境变量和 keyring | 环境变量优先 |
| Docker 容器环境 | 仅支持环境变量 |
| Keyring 读取返回空字符串 | 视为未设置，继续下一级 |
| Agent 未设置 Key 且 Provider 也未设置 | 报错，提示用户设置 |
| 同一 Agent 的 Keyring 和配置文件都有 Key | Keyring 优先，配置文件中的视为废弃 |
| 删除 Provider Key 但仍有 Agent 使用 | 仅删除 Provider 条目，Agent 不受影响 |

---

## 8. 实施计划

### Phase 1: 基础设施（0.5 天）
1. 添加 `keyring` + `rpassword` 依赖
2. 创建 `keyring.rs` 模块
3. 实现多 Provider/Agent 的 store/get/delete 函数
4. 实现 `is_keyring_available()` 和桌面环境检测

### Phase 2: Config 集成（1 天）
5. 修改 `Config.api_key` 和 `AgentConfig.api_key` 为 `Option<String>`
6. 实现 `resolve_api_key_for()` 5 级优先级链
7. 修改 `Config::save()` 排除 keyring 存储的 Key
8. 添加文件权限 0600 设置
9. 实现自动迁移逻辑

### Phase 3: CLI 增强（1 天）
10. 实现 `cmd_update_key()` 交互式流程
11. 实现 `cmd_delete_key()` 交互式流程
12. 实现 `cmd_list_keys()` 状态查看
13. 实现 `read_hidden_input()`

### Phase 4: 测试与文档（0.5 天）
14. 单元测试（keyring mock + 优先级链）
15. 集成测试（多 Agent Key 解析）
16. 更新 `config.example.toml` 注释
17. 更新 README 配置说明

---

## 9. 验收标准

- [ ] `cargo run` 在无 keyring 环境下能正常启动（文件回退）
- [ ] `cargo run` 在有 keyring 环境下能正常启动（keyring 优先）
- [ ] 不同 Agent 使用不同 Provider 时，各自 Key 正确解析
- [ ] Agent 专属 Key 覆盖 Provider 默认 Key
- [ ] `I_RS_CLAW_API_KEY` 和 `I_RS_CLAW_API_KEY_ANALYST` 环境变量始终优先
- [ ] `i-rs-claw config --update-key` 能安全输入并存储 Key
- [ ] `i-rs-claw config --list-keys` 能正确显示 Key 存储状态
- [ ] 配置文件权限自动设置为 0600
- [ ] 明文 Key 自动迁移到 keyring 后，配置文件不再包含明文
- [ ] 所有现有测试通过
- [ ] 新增 10+ 个单元测试
