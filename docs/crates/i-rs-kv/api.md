# i-rs-kv API

i-rs-kv 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 kv 端点均在 `/kv` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/kv` | 列出所有条目 |
| GET | `/kv?tag=` | 按标签过滤 |
| GET | `/kv?pattern=` | 按 key/value 模式过滤 |
| GET | `/kv/search?q=` | 搜索条目 |
| GET | `/kv/stats` | 统计信息 |
| GET | `/kv/{key}` | 获取单个条目详情 |
| POST | `/kv/{key}` | 创建条目 |
| DELETE | `/kv/{key}` | 删除条目 |
| PATCH | `/kv/{key}` | 更新条目 |
| POST | `/kv/{key}/copy` | 复制条目 |
| PATCH | `/kv/{key}/rename` | 重命名条目 |

## 全局响应格式

所有成功响应统一格式：

```json
{
  "success": true,
  "data": ...,
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

列表端点额外包含 `meta.count`：

```json
{
  "success": true,
  "data": [...],
  "meta": {
    "count": 3,
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

错误响应：

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Key 'xxx' not found"
  },
  "meta": {
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

错误码：
- `NOT_FOUND` (404) — 条目不存在
- `CONFLICT` (409) — 条目已存在
- `BAD_REQUEST` (400) — 参数无效
- `SERVER_ERROR` (500) — 服务端错误

## 端点详情

### 列出条目

```
GET /kv
GET /kv?tag=production
GET /kv?tag=config&pattern=api
```

查询参数：
- `tag` — 按标签过滤（可选）
- `pattern` — 按 key 或 value 模糊匹配（可选）

示例响应：

```json
{
  "success": true,
  "data": [
    {
      "key": "api_key",
      "value": "sk-xxx",
      "tags": ["production"],
      "remark": [],
      "created_at": "2026-01-14 10:00:00",
      "updated_at": "2026-01-14 10:00:00"
    }
  ],
  "meta": {
    "count": 1,
    "timestamp": "2026-01-15T10:30:00Z",
    "version": "0.0.2"
  }
}
```

### 搜索条目

```
GET /kv/search?q=config
```

查询参数：
- `q` — 搜索关键词（必填，大小写不敏感，匹配 key/value/tag/remark）

### 统计信息

```
GET /kv/stats
```

示例响应：

```json
{
  "success": true,
  "data": {
    "total_entries": 5,
    "total_tags": 3,
    "total_remarks": 1,
    "total_value_bytes": 120,
    "avg_value_bytes": 24.0,
    "oldest_entry": "api-key-1",
    "newest_entry": "api-key-5"
  }
}
```

### 获取条目

```
GET /kv/{key}
```

路径参数：
- `key` — 条目键名

### 创建条目

```
POST /kv/{key}
Content-Type: application/json

{
  "value": "sk-xxx"
}
```

路径参数：
- `key` — 条目键名

请求体：
- `value` (必填) — 存储的值

### 更新条目

```
PATCH /kv/{key}
Content-Type: application/json

{
  "value": "new-value",
  "tag": ["production", "backend"]
}
```

路径参数：
- `key` — 条目键名

请求体（所有字段可选）：
- `value` — 更新值
- `tag` — 替换标签
- `remark` — 替换备注

### 删除条目

```
DELETE /kv/{key}
```

### 复制条目

```
POST /kv/{key}/copy
Content-Type: application/json

{
  "dst": "new-key"
}
```

路径参数：
- `key` — 源条目键名

请求体：
- `dst` (必填) — 目标键名

### 重命名条目

```
PATCH /kv/{key}/rename
Content-Type: application/json

{
  "new": "renamed-key"
}
```

路径参数：
- `key` — 当前键名

请求体：
- `new` (必填) — 新键名
