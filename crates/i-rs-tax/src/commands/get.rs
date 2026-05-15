use crate::presentation::output_item;
use crate::storage;
use clap::Args;
use i_rs_core::presentation::OutputFormat;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct GetArgs {
    #[arg(help = "税务记录名称")]
    pub name: String,
}

pub fn execute(args: &GetArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;

    let entry = crate::storage::get_entry(&store, &args.name).ok_or_else(|| {
        anyhow::anyhow!("税务记录 '{}' 不存在", args.name)
    })?;

    if matches!(*format, OutputFormat::Json) {
        let data = serde_json::json!({
            "name": entry.name,
            "tax_type": entry.tax_type,
            "amount": entry.amount,
            "date": entry.date.to_string(),
            "year": entry.year,
            "status": entry.status,
            "tags": entry.tags,
            "remark": entry.remark,
            "created_at": entry.created_at.to_rfc3339(),
            "updated_at": entry.updated_at.to_rfc3339()
        });
        let output = output_item(&data, *format);
        println!("{output}");
    } else {
        println!("\n{} {}\n", "税务记录:".cyan().bold(), entry.name.green());
        println!("{} {}", "  税种:".dimmed(), entry.tax_type);
        println!("{} {}", "  金额:".dimmed(), format!("{:.2}", entry.amount).green());
        println!("{} {}", "  日期:".dimmed(), entry.date);
        println!("{} {}", "  年度:".dimmed(), entry.year);
        println!("{} {}", "  状态:".dimmed(), entry.status);
        println!("{} {}", "  标签:".dimmed(), if entry.tags.is_empty() { "无".dimmed().to_string() } else { entry.tags.join(", ") });
        println!("{} {}", "  备注:".dimmed(), if entry.remark.is_empty() { "无".dimmed().to_string() } else { entry.remark.join(", ") });
        println!("{} {}", "  创建:".dimmed(), entry.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("{} {}", "  更新:".dimmed(), entry.updated_at.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}
