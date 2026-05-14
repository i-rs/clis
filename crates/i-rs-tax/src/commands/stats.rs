use crate::presentation::{format_stats, TaxStats};
use crate::storage;
use clap::Args;

#[derive(Args)]
pub struct StatsArgs {
    #[arg(short, long, help = "年度 (默认当前年度)")]
    pub year: Option<i32>,
    #[arg(short, long, help = "税种过滤 (personal/vat)")]
    pub tax_type: Option<String>,
    #[arg(long, default_value = "false")]
    pub json: bool,
}

pub fn execute(args: &StatsArgs) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    let year = args.year.unwrap_or_else(|| chrono::Utc::now().format("%Y").to_string().parse().unwrap());

    let mut stats = TaxStats::new(year);

    for entry in store.entries.values() {
        if entry.year != year {
            continue;
        }

        if let Some(ref tax_type) = args.tax_type {
            let tax_type_lower = tax_type.to_lowercase();
            let matches = match tax_type_lower.as_str() {
                "personal" | "个人所得税" => matches!(entry.tax_type, crate::models::TaxType::Personal),
                "vat" | "增值税" => matches!(entry.tax_type, crate::models::TaxType::Vat),
                _ => true,
            };
            if !matches {
                continue;
            }
        }

        stats.add(entry);
    }

    if args.json {
        println!("{}", serde_json::json!({
            "success": true,
            "data": {
                "year": stats.year,
                "personal_total": stats.personal_total,
                "vat_total": stats.vat_total,
                "total": stats.total
            }
        }));
    } else {
        println!("{}", format_stats(&stats));
    }

    Ok(())
}

pub fn run(args: &StatsArgs) {
    if let Err(_e) = execute(args) {
        std::process::exit(1);
    }
}
