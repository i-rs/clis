# i-rs-weight API

i-rs-weight 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 weight 端点均在 `/weight` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/weight` | 列出所有体重记录 |
| POST | `/weight` | 添加体重记录 |
| GET | `/weight/stats` | 统计信息 |
| GET | `/weight/{id}` | 获取指定记录 |
| PATCH | `/weight/{id}` | 更新指定记录 |
| DELETE | `/weight/{id}` | 删除指定记录 |

&gt; 注释：自 weight 0.0.3 版本起，记录由日期主键改为 UUID 主键，CRUD 路径参数统一使用记录 ID。

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
    "message": "Weight 'abc12345' not found"
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

### 列出记录

```
GET /weight
```

### 添加记录

```
POST /weight
Content-Type: application/json

{
  "weight": 72.5,
  "date": "2026-01-15",
  "remark": ["morning", "fasted"]
}
```

请求体：
- `weight` (必填) — 体重值 (kg)
- `date` (可选) — 日期，默认当天
- `remark` (可选) — 备注数组

### 统计信息

```
GET /weight/stats
```

示例响应：

```json
{
  "success": true,
  "data": {
    "count": 10
  }
}
```

### 获取记录

```
GET /weight/{id}
```

路径参数：
- `id` — 记录 ID（支持 UUID 短前缀）

### 更新记录

```
PATCH /weight/{id}
Content-Type: application/json

{
  "weight": 73.0,
  "remark": ["afternoon"]
}
```

路径参数：
- `id` — 记录 ID

请求体（所有字段可选）：
- `weight` — 更新体重值
- `remark` — 替换备注

### 删除记录

```
DELETE /weight/{id}
```
