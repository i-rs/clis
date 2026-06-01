# Dashboard / Gateway 认证 — 设计方案

> 状态: Draft  
> 日期: 2026-05-20  
> 范围: i-rs-claw Dashboard + Gateway  
> 优先级: P0（安全底线）

---

## 1. 背景

### 1.1 现状

**Dashboard**（Axum Web 服务器）:
- 绑定 `127.0.0.1:3000`（默认）
- **无任何认证**，任何人访问即可：
  - 发送消息（消耗 API 额度）
  - 查看所有会话历史
  - 查看/修改配置（含 API Key 信息）
  - 管理 Agent
- 如果用户修改 host 为 `0.0.0.0`，暴露到网络则完全无防护

**Gateway**（Telegram / WeChat 接入）:
- Telegram Bot 通过 Webhook 接收消息
- WeChat 通过轮询接收消息
- **无签名验证**，任何人知道 Webhook URL 即可伪造消息

### 1.2 威胁模型

| 威胁 | 影响 | 可能性 |
|------|------|--------|
| 局域网用户访问 Dashboard | 消耗 API 额度、查看隐私数据 | 高 |
| 公网暴露 Dashboard | 任意人使用你的 AI 助理 | 中 |
| 伪造 Telegram Webhook | 向你的 Agent 发送恶意消息 | 低 |
| 伪造 WeChat 消息 | 同上 | 低 |
| CSRF 攻击 | 通过恶意网页操作你的 Dashboard | 中 |

### 1.3 目标

- Dashboard 所有 API 端点（除 health/login 外）需要认证
- Gateway Webhook 需要签名验证
- 支持 Token 认证（简单、适合个人使用）
- 支持可选的 Basic Auth（适合简单部署）
- 不引入 OAuth/JWT 等复杂方案（个人工具不需要）
- 零配置默认安全（默认 localhost + 自动生成 Token）

---

## 2. 架构设计

### 2.1 认证方式选择

**推荐: Bearer Token + 可选 Basic Auth**

| 方案 | 适用场景 | 复杂度 | 本项目采用 |
|------|---------|--------|-----------|
| Bearer Token | Dashboard API / SPA 前端 | 低 | ✅ 主方案 |
| Basic Auth | 简单浏览器访问 / curl | 低 | ✅ 可选 |
| JWT | 多用户 / 分布式 | 高 | ❌ 不需要 |
| OAuth2 | 第三方登录 | 高 | ❌ 不需要 |
| Session/Cookie | 多用户 Web 应用 | 中 | ❌ 不需要 |

**理由**: i-rs-claw 是个人工具，单用户场景。Bearer Token 足够安全且实现简单。

### 2.2 Token 生成策略

```
首次启动 Dashboard 时:
1. 检查配置中是否已设置 auth_token
2. 如未设置，自动生成 32 字节随机 Token（hex 编码 = 64 字符）
3. 写入配置文件 [dashboard].auth_token
4. 启动时打印: "Dashboard Token: <token>"
5. 用户可通过 `i-rs-claw dashboard --show-token` 查看
```

```rust
fn generate_auth_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
```

### 2.3 认证流程

```
客户端请求
    │
    ▼
┌─────────────────────────────┐
│  Auth Middleware             │
├─────────────────────────────┤
│  1. 检查是否公开端点          │
│     (health, login)          │
│     → 是 → 放行              │
│     → 否 → 继续              │
├─────────────────────────────┤
│  2. 检查 Authorization header│
│     Bearer <token>           │
│     → 匹配 → 放行             │
│     → 不匹配 → 继续           │
├─────────────────────────────┤
│  3. 检查 Basic Auth header   │
│     Basic base64(user:pass)  │
│     → 匹配 → 放行             │
│     → 不匹配 → 继续           │
├─────────────────────────────┤
│  4. 检查查询参数 ?token=xxx  │
│     （仅用于 SSE 流式端点）    │
│     → 匹配 → 放行             │
│     → 不匹配 → 401           │
└─────────────────────────────┘
```

---

## 3. 数据模型

### 3.1 配置变更

```rust
// config.rs
pub struct DashboardConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    
    /// Authentication token for API access.
    /// If None, a random token is generated on first start.
    #[serde(default)]
    pub auth_token: Option<String>,
    
    /// Optional basic auth credentials (username:password).
    /// If set, Basic Auth is accepted in addition to Bearer token.
    #[serde(default)]
    pub basic_auth: Option<BasicAuthConfig>,
    
    /// CORS allowed origins. Empty = only localhost allowed.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuthConfig {
    pub username: String,
    pub password: String,  // 可存储 bcrypt hash，但个人工具明文也可接受
}
```

### 3.2 Gateway 配置变更

```rust
// config.rs
pub struct GatewayConfig {
    pub enabled: bool,
    
    /// Webhook secret for signature verification.
    /// For Telegram: used to verify X-Telegram-Bot-Api-Secret-Token header.
    /// For WeChat: used to verify message signature.
    #[serde(default)]
    pub webhook_secret: Option<String>,
    
    pub telegram: Option<PlatformConfig>,
    pub wechat: Option<WeChatPlatformConfig>,
}
```

---

## 4. 模块设计

### 4.1 新增文件

```
crates/claw/src/
├── config.rs              — 修改（新增 auth 字段）
├── auth/
│   ├── mod.rs             — 模块入口
│   ├── middleware.rs       — Axum 认证中间件
│   ├── token.rs            — Token 生成/验证
│   └── webhook.rs          — Webhook 签名验证
└── dashboard/
    ├── mod.rs             — 修改（挂载认证中间件）
    └── routes.rs          — 修改（login 端点）
```

### 4.2 auth/middleware.rs — Axum 认证中间件

```rust
//! Axum middleware for Dashboard API authentication.
//!
//! Supports:
//! - Bearer Token (Authorization: Bearer <token>)
//! - Basic Auth (Authorization: Basic base64(user:pass))
//! - Query parameter (?token=xxx) for SSE endpoints only

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::config::DashboardConfig;

/// Paths that do not require authentication.
const PUBLIC_PATHS: &[&str] = &[
    "/api/health",
    "/api/login",
];

/// Authenticate a request.
pub async fn auth_middleware(
    config: DashboardConfig,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // Allow public paths
    if PUBLIC_PATHS.contains(&path) {
        return Ok(next.run(request).await);
    }
    
    // Try Bearer Token
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if verify_token(&config, token) {
                    return Ok(next.run(request).await);
                }
            }
        }
    }
    
    // Try Basic Auth
    if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(encoded) = auth_str.strip_prefix("Basic ") {
                if verify_basic_auth(&config, encoded) {
                    return Ok(next.run(request).await);
                }
            }
        }
    }
    
    // Try query parameter (only for SSE streaming endpoints)
    if path.starts_with("/api/chat/stream") {
        if let Some(query) = request.uri().query() {
            if let Some(token) = query
                .split('&')
                .find(|p| p.starts_with("token="))
                .map(|p| &p[6..])
            {
                if verify_token(&config, token) {
                    return Ok(next.run(request).await);
                }
            }
        }
    }
    
    // All checks failed
    Err(StatusCode::UNAUTHORIZED)
}

fn verify_token(config: &DashboardConfig, token: &str) -> bool {
    if let Some(ref expected) = config.auth_token {
        token == expected
    } else {
        false
    }
}

fn verify_basic_auth(config: &DashboardConfig, encoded: &str) -> bool {
    use base64::Engine;
    
    let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
        Ok(b) => b,
        Err(_) => return false,
    };
    
    let credentials = match String::from_utf8(decoded) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    if let Some(ref basic) = config.basic_auth {
        let expected = format!("{}:{}", basic.username, basic.password);
        credentials == expected
    } else {
        false
    }
}
```

### 4.3 auth/token.rs — Token 管理

```rust
//! API token generation and management.

use rand::Rng;

/// Generate a cryptographically secure random token (64 hex chars).
pub fn generate_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Ensure a token exists in the config, generating one if missing.
/// Returns (token, was_generated).
pub fn ensure_token(config: &mut crate::config::DashboardConfig) -> (String, bool) {
    if let Some(ref token) = config.auth_token {
        if !token.is_empty() {
            return (token.clone(), false);
        }
    }
    
    let new_token = generate_token();
    config.auth_token = Some(new_token.clone());
    (new_token, true)
}
```

### 4.4 auth/webhook.rs — Webhook 签名验证

```rust
//! Webhook signature verification for Gateway platforms.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify Telegram webhook signature.
/// Telegram sends X-Telegram-Bot-Api-Secret-Token header.
pub fn verify_telegram_webhook(
    secret: &str,
    received_token: Option<&str>,
) -> bool {
    match received_token {
        Some(token) => token == secret,
        None => false,
    }
}

/// Verify WeChat message signature.
/// WeChat sends msg_signature parameter.
pub fn verify_wechat_signature(
    token: &str,
    timestamp: &str,
    nonce: &str,
    msg_encrypt: &str,
    received_signature: &str,
) -> bool {
    // WeChat signature verification:
    // 1. Sort [token, timestamp, nonce, msg_encrypt] alphabetically
    // 2. Concatenate
    // 3. SHA1 hash
    // 4. Compare with received signature
    let mut parts = vec![token, timestamp, nonce, msg_encrypt];
    parts.sort();
    let concatenated = parts.concat();
    
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(concatenated.as_bytes());
    let computed = format!("{:x}", hasher.finalize());
    
    computed == received_signature
}

/// Compute HMAC-SHA256 signature for generic webhook verification.
pub fn compute_hmac_signature(secret: &str, payload: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload);
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Verify HMAC-SHA256 signature.
pub fn verify_hmac_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    let computed = compute_hmac_signature(secret, payload);
    computed == signature
}
```

### 4.5 dashboard/mod.rs — 挂载认证中间件

```rust
// 修改 router 构建，挂载认证中间件
pub fn build_router(state: AppState) -> Router {
    let config = state.config.clone();
    let dashboard_config = config.dashboard.clone();
    
    // Ensure token exists
    let (token, was_generated) = ensure_token(&mut state.config.dashboard);
    if was_generated {
        tracing::info!("Dashboard auth token auto-generated: {}", token);
        // 保存配置
        let _ = state.config.save();
    }
    
    Router::new()
        // Public routes
        .route("/api/health", get(routes::health))
        .route("/api/login", post(routes::login))
        // Protected routes (require auth)
        .route("/api/config", get(routes::get_config))
        .route("/api/message", post(routes::send_message))
        .route("/api/chat/stream", post(routes::chat_stream))
        .route("/api/session/current", get(routes::get_current_session))
        .route("/api/session/create", post(routes::create_session))
        // ... 其他路由
        .with_state(state)
        .layer(middleware::from_fn(move |req, next| {
            let config = dashboard_config.clone();
            auth_middleware(config, req, next)
        }))
}
```

### 4.6 dashboard/routes.rs — 新增 login 端点

```rust
/// Login endpoint — verifies credentials and returns the auth token.
/// This is a simple endpoint for SPA frontend authentication.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    let config = &state.config.dashboard;
    
    // Verify token
    if let Some(ref expected) = config.auth_token {
        if req.token == *expected {
            return Json(ApiResponse::ok(LoginResponse {
                success: true,
                message: Some("认证成功".to_string()),
            }));
        }
    }
    
    // Verify basic auth credentials
    if let Some(ref basic) = config.basic_auth {
        if req.username.as_deref() == Some(&basic.username)
            && req.password.as_deref() == Some(&basic.password)
        {
            return Json(ApiResponse::ok(LoginResponse {
                success: true,
                message: Some("认证成功".to_string()),
                token: config.auth_token.clone(),
            }));
        }
    }
    
    Json(ApiResponse::err("认证失败"))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: Option<String>,
    pub token: Option<String>,
}
```

---

## 5. Gateway 认证

### 5.1 Telegram Webhook

Telegram 支持设置 `secret_token`，每次 Webhook 回调会携带 `X-Telegram-Bot-Api-Secret-Token` header。

```rust
// gateway/telegram.rs
pub async fn handle_webhook(
    State(state): State<GatewayState>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    // Verify secret token
    let secret = state.config.gateway.webhook_secret
        .as_deref()
        .unwrap_or("");
    
    let received = headers
        .get("X-Telegram-Bot-Api-Secret-Token")
        .and_then(|v| v.to_str().ok());
    
    if !crate::auth::webhook::verify_telegram_webhook(secret, received) {
        tracing::warn!("[Telegram] Webhook signature verification failed");
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    // ... 处理消息
}
```

### 5.2 WeChat 签名验证

WeChat 使用 SHA1 签名验证：

```rust
// gateway/wechat.rs
pub async fn handle_message(
    State(state): State<GatewayState>,
    Query(params): Query<WeChatParams>,
    body: String,
) -> impl IntoResponse {
    let token = state.config.gateway.webhook_secret
        .as_deref()
        .unwrap_or("");
    
    if !crate::auth::webhook::verify_wechat_signature(
        token,
        &params.timestamp,
        &params.nonce,
        &body,
        &params.msg_signature,
    ) {
        tracing::warn!("[WeChat] Message signature verification failed");
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    // ... 处理消息
}
```

---

## 6. CORS 安全

### 6.1 默认策略

```rust
// 默认只允许 localhost 来源
fn default_allowed_origins() -> Vec<String> {
    vec![
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
    ]
}
```

### 6.2 配置覆盖

```toml
[dashboard]
allowed_origins = ["https://my-dashboard.example.com"]
```

### 6.3 中间件集成

```rust
use tower_http::cors::{CorsLayer, Any, Origin};

fn build_cors(config: &DashboardConfig) -> CorsLayer {
    if config.allowed_origins.is_empty() {
        // Default: only localhost
        CorsLayer::new()
            .allow_origin(Origin::list([
                "http://localhost:3000".parse().unwrap(),
                "http://127.0.0.1:3000".parse().unwrap(),
            ]))
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    } else {
        // Custom origins
        let origins: Vec<_> = config.allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(Origin::list(origins))
            .allow_methods(Any)
            .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    }
}
```

---

## 7. 配置示例

```toml
# ~/.i-rs/claw/config.toml

# ── Dashboard ───────────────────────────────────────────────────────────────
[dashboard]
enabled = true
host = "127.0.0.1"
port = 3000

# 认证 Token（不设置则自动生成）
auth_token = "a1b2c3d4e5f6..."

# 可选 Basic Auth
[dashboard.basic_auth]
username = "admin"
password = "my-secret-password"

# CORS 允许来源（默认仅 localhost）
allowed_origins = ["http://localhost:3000"]

# ── Gateway ─────────────────────────────────────────────────────────────────
[gateway]
enabled = true

# Webhook 签名密钥（Telegram secret_token / WeChat token）
webhook_secret = "my-webhook-secret-token"

[gateway.telegram]
enabled = true
token = "bot123456:ABC-DEF..."

[gateway.wechat]
enabled = true
```

---

## 8. CLI 增强

### 8.1 查看 Dashboard Token

```bash
$ i-rs-claw dashboard --show-token

Dashboard Token: a1b2c3d4e5f6...
API URL: http://127.0.0.1:3000

使用方式:
  curl -H "Authorization: Bearer a1b2c3d4e5f6..." http://127.0.0.1:3000/api/health
```

### 8.2 重新生成 Token

```bash
$ i-rs-claw dashboard --rotate-token

✓ Dashboard Token 已重新生成
新 Token: f6e5d4c3b2a1...
```

### 8.3 生成 Webhook Secret

```bash
$ i-rs-claw gateway --generate-secret

✓ Gateway Webhook Secret 已生成
Secret: wh_sec_abc123...

请将此 Secret 配置到 Telegram Bot 的 webhook 设置中。
```

---

## 9. 前端集成

### 9.1 SPA 前端认证流程

```
1. 用户打开 Dashboard (http://localhost:3000)
2. 前端检查 localStorage 中是否有 token
3. 如无 → 显示登录页面
4. 用户输入 Token 或用户名/密码
5. 前端调用 POST /api/login
6. 认证成功 → 存储 token 到 localStorage
7. 后续所有请求携带 Authorization: Bearer <token>
```

### 9.2 SSE 流式认证

由于 `EventSource` API 不支持自定义 Header，SSE 端点支持查询参数认证：

```javascript
const token = localStorage.getItem('dashboard_token');
const eventSource = new EventSource(
  `/api/chat/stream?token=${token}`
);
```

**安全说明**: 查询参数认证仅用于 SSE（浏览器限制），其他端点必须使用 Header。

---

## 10. 边界情况处理

| 场景 | 处理方式 |
|------|---------|
| Token 泄露 | `--rotate-token` 重新生成，旧 Token 立即失效 |
| 忘记 Token | `--show-token` 查看，或查看启动日志 |
| 配置文件无 auth_token | 首次启动自动生成 |
| Basic Auth 密码明文 | 个人工具可接受，未来可加 bcrypt |
| SSE token 出现在日志中 | 日志中脱敏显示 `token=****` |
| CORS 预检请求 (OPTIONS) | 中间件放行，不消耗认证 |
| 多个 Dashboard 实例 | 每个实例独立 Token |
| Gateway webhook_secret 未设置 | 跳过签名验证 + warn 日志 |

---

## 11. 依赖变更

```toml
[dependencies]
# 新增
rand = "0.8"
hmac = "0.12"
sha2 = "0.10"
sha1 = "0.10"
hex = "0.4"
```

---

## 12. 实施计划

### Phase 1: 基础设施（0.5 天）
1. 添加依赖（rand, hmac, sha2, sha1, hex）
2. 创建 `auth/` 模块
3. 实现 `token.rs`（Token 生成/验证）
4. 实现 `middleware.rs`（Axum 认证中间件）

### Phase 2: Dashboard 集成（1 天）
5. 修改 `DashboardConfig` 新增 auth 字段
6. 挂载认证中间件到 Router
7. 新增 `/api/login` 端点
8. 新增 CORS 中间件
9. 实现 `--show-token` / `--rotate-token` CLI

### Phase 3: Gateway 集成（1 天）
10. 修改 `GatewayConfig` 新增 webhook_secret
11. 实现 Telegram Webhook 签名验证
12. 实现 WeChat 消息签名验证
13. 实现 `--generate-secret` CLI

### Phase 4: 测试与文档（0.5 天）
14. 单元测试（Token 生成/验证、签名验证）
15. 集成测试（认证中间件）
16. 更新 `config.example.toml`
17. 更新 README

---

## 13. 验收标准

- [ ] Dashboard 启动时自动生成 Token（如未配置）
- [ ] 无 Token 的请求返回 401
- [ ] Bearer Token 认证通过
- [ ] Basic Auth 认证通过（如配置）
- [ ] `/api/health` 和 `/api/login` 无需认证
- [ ] SSE 端点支持 `?token=` 查询参数
- [ ] CORS 默认仅允许 localhost
- [ ] `--show-token` 显示当前 Token
- [ ] `--rotate-token` 重新生成 Token
- [ ] Telegram Webhook 签名验证生效
- [ ] WeChat 消息签名验证生效
- [ ] 所有现有测试通过
- [ ] 新增 10+ 个单元测试
