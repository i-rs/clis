# i-rs-meal API

i-rs-meal 提供 REST API，可通过 `i-rs-api` 服务器访问。

## 启动 API 服务

```bash
i-rs-api
```

默认监听 `http://0.0.0.0:8080`，所有 meal 端点均在 `/meal` 路径下。

## 端点概览

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/meal` | 列出所有饮食记录 |
| POST | `/meal` | 添加饮食记录 |
| GET | `/meal/{id}` | 获取单条记录 |
| DELETE | `/meal/{id}` | 删除记录 |
| PATCH | `/meal/{id}` | 更新记录 |

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
    "message": "Meal 'uuid-xxx' not found"
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
GET /meal
```

### 添加记录

```
POST /meal
Content-Type: application/json

{
  "meal_type": "breakfast",
  "food_items": "oatmeal, banana, coffee",
  "calories": 350,
  "tags": ["healthy"],
  "remark": [],
  "date": "2026-01-15"
}
```

请求体：
- `meal_type` (必填) — 餐型 (如 breakfast, lunch, dinner, snack)
- `food_items` (必填) — 食物描述
- `date` (必填) — 日期，格式 `YYYY-MM-DD`
- `calories` (可选) — 卡路里估算
- `tags` (可选) — 标签数组
- `remark` (可选) — 备注数组

### 获取记录

```
GET /meal/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID

### 更新记录

```
PATCH /meal/{id}
Content-Type: application/json

{
  "calories": 400
}
```

路径参数：
- `id` — UUID 格式的条目 ID

请求体（所有字段可选）：
- `meal_type` — 更新餐型
- `food_items` — 更新食物描述
- `calories` — 更新卡路里
- `tags` — 替换标签
- `remark` — 替换备注
- `date` — 更新日期

### 删除记录

```
DELETE /meal/{id}
```

路径参数：
- `id` — UUID 格式的条目 ID
