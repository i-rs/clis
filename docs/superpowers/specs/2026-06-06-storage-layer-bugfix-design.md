# claw 存储层关键 Bug 修复设计

**日期:** 2026-06-06
**来源:** `STORAGE_REVIEW.md` 审查报告

## 范围

Phase 1 关键正确性修复 + SessionRepo trait 重构，共 6 项：

| # | 问题 | 文件 | 改动规模 |
|---|------|------|----------|
| 4 | `index.json` 损坏时静默清空 | `storage/file.rs` | ~15 行 |
| 5 | `StatsRepo::prune` 缺锁 | `storage/file.rs` | ~5 行 |
| 6 | `append_new_messages` 不更新 `message_count` | `session.rs` | ~5 行 |
| 3 | SQL `load` 查询逻辑错误 | `storage/sql/mod.rs` | ~15 行 |
| 7+8 | SessionRepo trait 重构 (upsert/delete/get) | `storage/mod.rs` + 所有后端 | ~200 行 |

## 设计决策

### #4 index.json 损坏保护
- `load_all` 损坏时返回 `Err` 而非空 `Vec`
- `save_all` 前备份旧文件为 `index.json.bak`
- 不自动重建 index (恢复操作留给专用命令，超出当前范围)

### #5 StatsRepo::prune 加锁
- 在 `prune` 的 `blocking` 闭包开头加 `STATS_LOCK` guard
- 与 `upsert_batch` 保持一致

### #6 append_new_messages 更新 message_count
- 在 `Ok(())` 分支增加 `session_meta.message_count += new_msgs.len()`
- 调用 `save_index()` 持久化

### #3 SQL load 改用子查询 + LIMIT
- 原: `WHERE seq > MAX(seq) - limit` (逻辑错误)
- 改: `SELECT ... FROM (SELECT ... ORDER BY seq DESC LIMIT ?) sub ORDER BY seq ASC`

### #7+#8 SessionRepo trait 重构
- 在 trait 中添加 `upsert`, `delete_one`, `get_one`, `count` 为必需方法
- 为 file 后端保留 `save_all`/`load_all` 作为内部实现
- SQL 后端: 单行 INSERT ... ON CONFLICT / SELECT / DELETE
- Mongo: `update_one` with upsert / `find_one` / `delete_one`
- Redis: HSET / HGET / HDEL
- 删除 `get`/`delete`/`count` 的默认实现

## 验证计划
1. `cargo check` — 0 errors, 0 warnings
2. `cargo test -p i-rs-claw -- --test-threads=1` — 206 tests pass
3. `cargo clippy -p i-rs-claw -- -D warnings` — 清洁
