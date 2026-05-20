---
description = "测试技能: 将用户输入回显为指定格式"

[parameters]
type = "object"
required = ["message"]

[parameters.properties]
message = { type = "string", description = "要回显的消息" }
style = { type = "string", enum = ["plain", "shout", "whisper"], description = "回显风格" }
---

# Test Echo Skill

当用户调用此技能时，根据指定的 `style` 回显 `message`：

- **plain**: 原样返回 `用户: {message}`
- **shout**: 大写返回 `注意: {MESSAGE}！`
- **whisper**: 小写返回 `悄悄说: {message}...`

此技能用于验证 i-rs-claw 的 skill 系统是否正确工作。
