# 一个人的数据指挥中心 — 全领域覆盖

## 你的生活，需要多少工具？

试试数一数你生活中需要记录和管理的事情：

- **健康** — 体重、身高、睡眠、运动、饮水、心率、视力
- **财务** — 流水、预算、投资、债务、发票、税务、储蓄目标
- **家庭** — 家电维护、床单更换、牙刷更换、净水器滤芯
- **宠物** — 喂食、洗澡、遛狗、水族箱维护
- **学习** — 阅读、词汇量、文章收藏、灵感捕捉
- **社交** — 联系人、生日提醒、礼物规划
- **娱乐** — 电影、播客、名言收藏
- **数字** — 密码、API 密钥、笔记、书签、代码片段
- **时间** — 事件、定时提醒、番茄钟、部署记录

对于每一项，你很可能已经在用不同的 App、不同的网站、不同的记事本。但你的数据被锁在各自的孤岛中，互不相通。

i-rs 的回答是：**一个终端，全部管理。**

## 70 个工具，一个生态

i-rs 的 70 个 CLI 工具，覆盖了个人生活的几乎所有可数据化的领域：

```
健康管理 ◄──── i-rs-weight  i-rs-height  i-rs-sleep  i-rs-mood
              i-rs-exercise  i-rs-run  i-rs-cycling  i-rs-water
              i-rs-step  i-rs-dose  i-rs-fast  i-rs-allergy
              i-rs-cal  i-rs-habit  i-rs-cycle  i-rs-sit

财务管理 ◄──── i-rs-ledger  i-rs-budget  i-rs-invest  i-rs-debt
              i-rs-recur  i-rs-invoice  i-rs-tax  i-rs-goal  i-rs-sub

家庭维护 ◄──── i-rs-appliance  i-rs-sheet  i-rs-toothbrush
              i-rs-towel  i-rs-bed  i-rs-filter  i-rs-purify
              i-rs-ac  i-rs-plant  i-rs-aqua

宠物照护 ◄──── i-rs-feedpet  i-rs-petbath  i-rs-walkdog

学习记录 ◄──── i-rs-read  i-rs-vocab  i-rs-article  i-rs-spark

社交关系 ◄──── i-rs-contact  i-rs-birthday  i-rs-gift

娱乐收藏 ◄──── i-rs-movie  i-rs-podcast  i-rs-quote

数字资产 ◄──── i-rs-password  i-rs-keys  i-rs-kv
              i-rs-bookmark  i-rs-note  i-rs-snippet

时间管理 ◄──── i-rs-remind  i-rs-event  i-rs-time
              i-rs-tick  i-rs-deploy

其他工具 ◄──── i-rs-domain  i-rs-bestby  i-rs-todo
              i-rs-project  i-rs-grocery  i-rs-meal
              i-rs-pig  i-rs-want  i-rs-vision
              i-rs-car i-rs-sit
```

这不是"又一个 TODO 应用"。这是**一个人对自己生活所有维度的数据化掌控**。

## 命令之美

每个工具都遵循一致的命令风格，学习成本趋近于零：

```bash
# 记录今天的体重
i-rs-weight add --weight 72.5

# 查看本月支出
i-rs-ledger list --month 2026-05

# 检查宠物喂食记录
i-rs-feedpet list

# 统计投资回报率
i-rs-invest stats

# 查看哪些食物快过期了
i-rs-bestby list

# 今天学习了什么
i-rs-vocab quiz
```

同样的 `add`、`list`、`get`、`update`、`delete` 模式贯穿所有工具，一旦上手一个，就会用全部。

## 数据在你手中

i-rs 不做云同步，不绑架你的数据。所有数据存储在本地 `~/.config/i-rs/` 目录下，纯 JSON 格式。

这意味着：

- **离线可用** — 不需要网络连接
- **完全掌控** — 你可以 `cat`、`grep`、`rsync` 你的数据
- **任意备份** — 一个 `cp -r` 就完成了全量备份
- **版本可控** — 配合 git 即可实现数据版本管理
- **格式开放** — 纯 JSON，任何语言都能读取

```bash
# 备份所有 i-rs 数据
cp -r ~/.config/i-rs ~/backups/i-rs-$(date +%Y%m%d)

# 统计分析（用任何你喜欢的方式）
cat ~/.config/i-rs/weight.json | jq '.entries | length'
```

## 从数据记录到数据洞察

很多工具止步于"记录"。i-rs 更进一步：

- **`stats`** 子命令 — 许多工具内置统计分析
- **`chart`** 子命令 — i-rs-weight 可直接生成 ASCII 图表
- **`--json`** 全局标志 — 所有命令支持 JSON 输出，方便进一步处理
- **`data export / import / clear`** — 完整的数据生命周期管理

```bash
# 获取体重统计
i-rs-weight stats
# Total records: 180
# Average: 72.3 kg
# Min: 68.5 kg (2026-01-15)
# Max: 75.2 kg (2026-03-20)
# Trend: -2.9 kg over 6 months
```

## 为什么是 CLI？

在 GUI 应用泛滥的今天，为什么选择命令行？

这不是复古，而是效率。CLI 的优势在于：

1. **键盘驱动** — 双手不离键盘，操作速度是鼠标的 3-5 倍
2. **脚本化** — `alias`、`cron`、快捷键，极致自动化
3. **低开销** — 一个二进制文件，几毫秒启动，几 KB 内存
4. **远程友好** — SSH 会话中完美工作，无需图形环境
5. **组合性** — 管道、重定向、子 shell，创造无限可能
6. **一致性** — 70 个工具，同一套语法，零学习曲线

## 属于你的数据指挥中心

想象一下：

- 清早 `i-rs-sleep stats` 查看睡眠质量
- 中午 `i-rs-meal add` 记录午餐
- 下午 `i-rs-project list` 检查项目进度
- 睡前 `i-rs-mood add --mood great` 记录心情
- 每分钟平均耗时不到一秒

这就是 i-rs 想要实现的：**用最小的心智负担，完成最全面的生活记录。**

这不是一个工具，这是一个属于你的**个人数据指挥中心**。
