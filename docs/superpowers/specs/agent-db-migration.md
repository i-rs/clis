# Agent + Config 数据库迁移 Spec

**日期**: 2026-06-07 | **状态**: Draft

## 1. 动机

当前 `config.toml` 管理大量运行时数据（agent/provider/MCP/dashboard/gateway/stats 等），存在问题：

- **多用户不隔离** — 所有用户的 agent/provider 配置共享一份 config.toml
- **API 半成品** — agents 可通过 API 创建/修改，但回写 config.toml，并发不安全
- **文件-SQL 不一致** — File backend 的结构和 SQL backend 不对齐（如 agnet 配置只存在文件里）
- **不能动态热载** — 大部分配置需重启生效

## 2. 迁移范围决策

### 2.1 移到数据库

| 数据 | 当前位置 | 原因 |
|------|---------|------|
| **Agent 配置** | config.toml `[agents.{id}]` | 多用户隔离、API CRUD 并发安全 |
| **Provider 配置** | config.toml `[providers.{name}]` | 多用户共享或多用户独立 |
| **Dashboard users** | config.toml `[[dashboard.users]]` | 多用户管理的自然归属 |
| **MCP server 配置** | config.toml `[[mcp_servers]]` | agent 级别的多用户配置 |
| **Quality judge 配置** | config.toml `[quality_judge]` | agent 级别差异 |
| **Stats 配置** | config.toml `[stats]` | 已在 token_records 表旁，合并合理 |
| **Image gen 配置** | config.toml `[image_gen]` | agent/provider 级别差异 |
| **Behavior analyst** | config.toml `[behavior_analyst]` | agent 级别差异 |

### 2.2 留在 config.toml（启动引导）

| 数据 | 原因 |
|------|------|
| **Storage backend** (`[storage]`) | 必须知道连哪个 DB 才能读其他配置 |
| **Dashboard host/port** (`[dashboard].host/port`) | HTTP 服务器启动前置 |
| **Gateway enabled** (`[gateway].enabled`) | 启动前置 |
| **Timezone** | 启动前置 |
| **Execution engine params** (max_react_rounds, cli_timeout_secs, etc.) | 全局不变，无须 DB |
| **Tool whitelists** (enabled_tools, i_rs_tools, allowed_dirs) | 全局不变 |
| **Plugins** (plugins_auto_discover, disabled_plugins) | 启动前置 |

### 2.3 已 DB 化的（不动）

Session, MessageLog, ApiCache, PlanSteps, Memory, Stats, Skills, ToolCache — 已在 `ClawStorage` 中。

## 3. 迁移后架构

```
~/.i-rs/claw/
├── config.toml              ← 仅引导配置 (< 30 行)
│   ├── [storage]             # 后端类型 + 连接参数
│   ├── [dashboard]           # host, port
│   ├── [gateway]             # enabled 开关
│   ├── timezone
│   ├── execution params
│   └── tool/i_rs whitelists
│
└── DB (任意 backend)
    ├── sessions / message_log / ...
    ├── agent_configs        ← 新表
    │   ├── user_id, agent_id, provider_ref, model, ...
    │   ├── enabled_tools (JSON array)
    │   ├── system_prompt, capabilities (JSON array)
    │   └── mcp_servers (JSON array)
    ├── provider_configs     ← 新表 (可选: 全局共享或 per-user)
    │   ├── user_id (NULL=global), name, provider, api_key, base_url, model
    ├── dashboard_users      ← 新表
    │   ├── user_id, token_hash, display_name, created_at
    ├── mcp_server_configs   ← 新表
    │   ├── user_id, agent_id (NULL=global), name, transport_type, ...
    └── app_settings         ← K/V 表用于 quality_judge/image_gen/stats/behavior_analyst
        ├── key (PK), value (JSON)
```

## 4. 数据模型

### 4.1 agent_configs

```sql
CREATE TABLE agent_configs (
    user_id          VARCHAR(64)  NOT NULL DEFAULT 'default',
    agent_id         VARCHAR(64)  NOT NULL,
    provider_ref     VARCHAR(128),
    provider         VARCHAR(32),
    api_key          TEXT,
    base_url         TEXT,
    model            VARCHAR(128),
    enabled_tools    TEXT,           -- JSON array
    system_prompt    TEXT,
    system_prompt_file TEXT,
    capabilities     TEXT,           -- JSON array
    execution_mode   VARCHAR(32),
    mcp_servers      TEXT,           -- JSON array
    allowed_dirs     TEXT,           -- JSON array
    created_at       BIGINT NOT NULL,
    updated_at       BIGINT NOT NULL,
    PRIMARY KEY (user_id, agent_id)
);
```

### 4.2 provider_configs

```sql
-- 方案 A: 全局共享（所有 user 共用 provider 定义）
CREATE TABLE provider_configs (
    name             VARCHAR(128) PRIMARY KEY,
    provider         VARCHAR(32)  NOT NULL,
    api_key          TEXT         NOT NULL DEFAULT '',
    base_url         TEXT         NOT NULL DEFAULT '',
    model            VARCHAR(128) NOT NULL DEFAULT '',
    created_at       BIGINT NOT NULL,
    updated_at       BIGINT NOT NULL
);

-- 方案 B: per-user（每个用户独立 provider）
CREATE TABLE provider_configs (
    user_id          VARCHAR(64)  NOT NULL DEFAULT 'default',
    name             VARCHAR(128) NOT NULL,
    provider         VARCHAR(32)  NOT NULL,
    api_key          TEXT         NOT NULL DEFAULT '',
    base_url         TEXT         NOT NULL DEFAULT '',
    model            VARCHAR(128) NOT NULL DEFAULT '',
    created_at       BIGINT NOT NULL,
    updated_at       BIGINT NOT NULL,
    PRIMARY KEY (user_id, name)
);
```

**推荐方案 A** — API key 通常是一个组织的资源，多用户共享 provider 连接池更合理。如有 per-user API key 需求，可通过 agent_configs 的 api_key 字段覆盖。

### 4.3 dashboard_users

```sql
CREATE TABLE dashboard_users (
    user_id          VARCHAR(64)  PRIMARY KEY,
    token_hash       VARCHAR(128) NOT NULL,  -- SHA-256 of token
    display_name     VARCHAR(128),
    created_at       BIGINT NOT NULL,
    updated_at       BIGINT NOT NULL
);
```

`auth_token`（全局单 token）保留在 config.toml 作为 bootstrap admin token。

### 4.4 mcp_server_configs

```sql
CREATE TABLE mcp_server_configs (
    user_id          VARCHAR(64)  NOT NULL DEFAULT 'default',
    agent_id         VARCHAR(64)  NULL,       -- NULL = global
    name             VARCHAR(128) NOT NULL,
    transport_type   VARCHAR(32)  NOT NULL DEFAULT 'stdio',
    command          TEXT,
    args             TEXT,        -- JSON array
    url              TEXT,
    env              TEXT,        -- JSON array
    enabled          SMALLINT     NOT NULL DEFAULT 1,
    PRIMARY KEY (user_id, COALESCE(agent_id, ''), name)
);
```

### 4.5 app_settings

```sql
CREATE TABLE app_settings (
    key              VARCHAR(255) PRIMARY KEY,
    value            TEXT NOT NULL,             -- JSON
    updated_at       BIGINT NOT NULL
);
```

存放: `quality_judge`, `image_gen`, `stats.default`, `behavior_analyst`, `default_provider`

## 5. 存储后端 DDL（所有 backend）

### 5.1 SQLite

```sql
CREATE TABLE agent_configs (...);
CREATE TABLE provider_configs (...);
CREATE TABLE dashboard_users (...);
CREATE TABLE mcp_server_configs (...);
CREATE TABLE app_settings (...);
```

### 5.2 MySQL

Same DDL, 使用 `VARCHAR(64) CHARACTER SET utf8mb4`，TEXT 字段用 `MEDIUMTEXT`，`ENGINE=InnoDB`。

### 5.3 PostgreSQL

Same DDL，`TEXT` 替代 `VARCHAR(255)`，`BIGINT` for timestamps。

### 5.4 MongoDB

```
collection: agent_configs
document: { _id: "{user_id}:{agent_id}", user_id, agent_id, provider_ref, ... }

collection: provider_configs  
document: { _id: name, provider, api_key, base_url, model, ... }

collection: dashboard_users
document: { _id: user_id, token_hash, display_name, ... }

collection: mcp_server_configs
document: { _id: "{user_id}:{agent_id||'global'}:{name}", ... }

collection: app_settings
document: { _id: key, value, ... }
```

### 5.5 Redis

```
claw:agent:{user_id}:{agent_id}         → HASH (所有 agent config 字段)
claw:provider:{name}                    → HASH
claw:dashboard:users                    → HASH (user_id → token_hash JSON)
claw:mcp:{user_id}:{agent_id||'g'}      → SET (server names)
claw:mcp:{...}:{name}                   → HASH (server config)
claw:settings                           → HASH (key → value JSON)
```

### 5.6 File

```
{claw_dir}/
├── agents/{user_id}/
│   └── agents.json                  ← HashMap<agent_id, AgentConfigRow>
├── providers/
│   └── providers.json               ← HashMap<name, ProviderConfigRow>
├── dashboard/
│   └── users.json                   ← HashMap<user_id, DashboardUserRow>
├── mcp/
│   └── {user_id}_{agent_id}.json    ← Vec<McpServerConfigRow>
└── settings.json                    ← HashMap<key, serde_json::Value>
```

## 6. 迁移流程

**无自动迁移。** 当前开发阶段直接断裂：

- `File` backend：保持 config.toml 读写，现有行为不变
- DB backend（SQLite/MySQL/PG/Mongo/Redis）：启动时直接从对应表读取，config.toml 中的 agent/provider/mcp 段**忽略**
- 用户首次切换到 DB backend 时**手动**配置

### config.toml 角色（精简后）

```
[storage]              ← 选 backend + 连接参数
[dashboard]            ← host, port
[gateway]              ← enabled 开关
timezone               ← 时区
execution params       ← max_react_rounds, cli_timeout 等
tool whitelists        ← enabled_tools, i_rs_tools, allowed_dirs
plugins                ← 自动发现 + 禁用列表

# DB backend 启动时完全忽略以下段:
# [agents], [providers], [[mcp_servers]], [quality_judge],
# [image_gen], [behavior_analyst], [stats], [[dashboard.users]]
```

## 7. API 适配

### 7.1 现有端点变更

| 端点 | 旧行为 | 新行为 |
|------|--------|--------|
| `GET /api/config` | 读 config.toml 脱敏部分 | 读 config.toml 引导部分 + DB agent/provider/dashboard |
| `PATCH /api/config` | 写 config.toml | 写 DB (provider + default model 等) |
| `POST /api/agents` | 写 config.toml + init runtime | INSERT agent_configs + init runtime |
| `PUT /api/agents/:id` | 修改 config.toml | UPDATE agent_configs |
| `DELETE /api/agents/:id` | 删除 config.toml entry | DELETE agent_configs |

### 7.2 新增端点

| 端点 | 描述 |
|------|------|
| `POST/PUT/DELETE /api/providers` | Provider CRUD |
| `GET/POST/DELETE /api/dashboard/users` | Dashboard 多用户管理 |
| `GET/PUT /api/settings` | app_settings K/V |
| `GET/POST/PUT/DELETE /api/mcp` | MCP server 配置管理 |

## 8. 多用户隔离

迁移后，AgentCRuntimeStore 的 key 已经改为 `(user_id, agent_id)`。数据库中的 agent_configs 天然按 `user_id` 隔离。

Dashboard users 表支持运行时创建/删除用户，token hash 用于认证对照。

## 9. 实施阶段

### Phase 1: DDL (0.5 天)
1. 所有 5 个 backend 增加 5 张新表的 DDL
2. 对应 File backend 的 JSON 文件存储实现

### Phase 2: 配置加载改造 (1 天)
1. `AppCore::with_claw_dir()` 检测 storage.backend
   - File → 从 config.toml 读 agent/provider/mcp
   - DB → 从对应表读取
2. `Config` 结构精简或拆分为 `BootstrapConfig` + `RuntimeConfig`

### Phase 3: API 层改造 (1-2 天)
1. 修改现有 agents/config 端点读写 DB（非 File backend 时）
2. 新增 provider/dashboard_users/settings/mcp 端点
3. 更新前端 Config/Agents 页面对接新 API

### Phase 4: 测试 (0.5 天)
1. 所有 backend 的基础 CRUD 测试
2. File backend 向后兼容验证

## 10. 不做

- 不迁移 tool whitelist（enabled_tools/i_rs_tools）— 全局属性，放 config.toml 足够
- 不迁移 execution params（max_react_rounds 等）— 同上
- 不迁移 theme — 已有独立 theme.json
- 不在 Phase 1 做 Redis/Mongo 的复杂索引 — 先让基础 DDL 跑通
