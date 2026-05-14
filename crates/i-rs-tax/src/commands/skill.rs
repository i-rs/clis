use clap::Args;

#[derive(Args)]
pub struct SkillArgs {
    #[arg(help = "查看方式 (summary/content)")]
    pub way: Option<String>,
}

const SKILL_CONTENT: &str = r#"---
name: "i-rs-tax"
description: "税务记录管理 CLI 工具。用于记录个人所得税、增值税等税务信息，支持年度统计和报税状态跟踪。"
---

# i-rs-tax 税务记录管理

税务记录管理 CLI 工具，用于记录个人所得税、增值税等税务信息。

## 存储

- 配置: `~/.config/i-rs/tax.json`

## 税种类型

- `personal` / `个人所得税`: 个人所得税
- `vat` / `增值税`: 增值税

## 报税状态

- `unreported` / `未申报`: 尚未申报
- `filing` / `申报中`: 申报中
- `filed` / `已申报`: 已申报
- `paid` / `已缴纳`: 已缴纳

## 命令

### add - 添加记录
```bash
i-rs-tax add <名称> --tax-type <税种> --amount <金额> --date <日期> [选项]
```

选项:
- `--tax-type, -t`: 税种类型 (personal/vat)
- `--amount, -a`: 金额
- `--date, -d`: 日期 (YYYY-MM-DD)
- `--year, -y`: 年度 (可选，默认从日期提取)
- `--status, -s`: 报税状态 (可选，默认 unreported)
- `--tag`: 标签 (可多次指定)
- `--remark`: 备注 (可多次指定)

示例:
```bash
i-rs-tax add 个税2024 -t personal -a 12000 -d 2024-03-15 -s filed --tag 工资
```

### list - 列出记录
```bash
i-rs-tax list [选项]
```

选项:
- `--tag`: 按标签过滤
- `--year`: 按年度过滤
- `--tax-type`: 按税种过滤

### get - 查看详情
```bash
i-rs-tax get <名称>
```

### delete - 删除记录
```bash
i-rs-tax delete <名称>
```

### stats - 年度统计
```bash
i-rs-tax stats [选项]
```

选项:
- `--year, -y`: 指定年度 (默认当前年度)
- `--tax-type, -t`: 按税种过滤

### example - 使用示例
```bash
i-rs-tax example
```

### skill - AI 技能文档
```bash
i-rs-tax skill [summary]
```

## 数据结构

```json
{
  "name": "个税2024",
  "tax_type": "个人所得税",
  "amount": 12000.00,
  "date": "2024-03-15",
  "year": 2024,
  "status": "已申报",
  "tags": ["工资"],
  "remark": [],
  "created_at": "2024-03-15T10:00:00Z",
  "updated_at": "2024-03-15T10:00:00Z"
}
```
"#;

const SKILL_SUMMARY: &str = r#"i-rs-tax: 税务记录管理工具

功能: 记录个人所得税、增值税，支持年度统计和报税状态跟踪
存储: ~/.config/i-rs/tax.json

命令:
- add: 添加税务记录 (--tax-type personal/vat, --amount, --date)
- list: 列出记录 (--tag, --year, --tax-type)
- get: 查看详情
- delete: 删除记录
- stats: 年度统计 (--year)
- example: 使用示例
- skill: AI 技能文档

税种: personal(个人所得税), vat(增值税)
状态: unreported, filing, filed, paid
"#;

pub fn execute(args: &SkillArgs) -> anyhow::Result<()> {
    match args.way.as_deref() {
        Some("summary") => {
            println!("{}", SKILL_SUMMARY);
        }
        Some("content") | None => {
            println!("{}", SKILL_CONTENT);
        }
        _ => {
            println!("{}", SKILL_CONTENT);
        }
    }
    Ok(())
}

pub fn run(args: &SkillArgs) {
    if let Err(e) = execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
