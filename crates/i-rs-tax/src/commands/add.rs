use crate::models::{TaxRecord, TaxStatus, TaxType};
use crate::presentation::print_success;
use crate::storage;
use chrono::Utc;
use clap::Args;
use i_rs_core::utils::validation::validate_name;

#[derive(Args)]
pub struct AddArgs {
    #[arg(help = "税务记录名称")]
    pub name: String,
    #[arg(short, long, help = "税种类型 (personal/vat)")]
    pub tax_type: String,
    #[arg(short, long, help = "金额")]
    pub amount: f64,
    #[arg(short, long, help = "日期 (YYYY-MM-DD)")]
    pub date: String,
    #[arg(short, long, help = "年度 (如: 2024)")]
    pub year: Option<i32>,
    #[arg(short, long, help = "报税状态 (unreported/filing/filed/paid)")]
    pub status: Option<String>,
    #[arg(short, long, help = "标签 (可多次指定)")]
    pub tag: Vec<String>,
    #[arg(short, long, help = "备注 (可多次指定)")]
    pub remark: Vec<String>,
    #[arg(long, default_value = "false")]
    pub json: bool,
}

pub fn execute(args: &AddArgs) -> anyhow::Result<()> {
    if let Err(e) = validate_name(&args.name) {
        anyhow::bail!("{}", e.message);
    }

    let tax_type = TaxType::from_str(&args.tax_type).ok_or_else(|| {
        let types = TaxType::variants().join(", ");
        anyhow::anyhow!("无效的税种类型 '{}'. 可用类型: {}", args.tax_type, types)
    })?;

    let date = chrono::NaiveDate::parse_from_str(&args.date, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("无效的日期格式 '{}'. 请使用 YYYY-MM-DD 格式", args.date))?;

    let year = args.year.unwrap_or_else(|| date.year());

    let status = match &args.status {
        Some(s) => TaxStatus::from_str(s).ok_or_else(|| {
            let statuses = TaxStatus::variants().join(", ");
            anyhow::anyhow!("无效的报税状态 '{s}'. 可用状态: {statuses}")
        })?,
        None => TaxStatus::Unreported,
    };

    let mut store = storage::load_store()?;

    if store.entries.contains_key(&args.name) {
        anyhow::bail!("税务记录 '{}' 已存在", args.name);
    }

    let now = Utc::now();
    let entry = TaxRecord {
        name: args.name.clone(),
        tax_type,
        amount: args.amount,
        date,
        year,
        status,
        tags: args.tag.clone(),
        remark: args.remark.clone(),
        created_at: now,
        updated_at: now,
    };

    storage::add_entry(&mut store, entry);
    storage::save_store(&store)?;

    if args.json {
        println!("{}", serde_json::json!({
            "success": true,
            "data": {
                "name": args.name,
                "tax_type": tax_type,
                "amount": args.amount,
                "date": date.to_string(),
                "year": year,
                "status": status,
                "tags": args.tag,
                "remark": args.remark
            }
        }));
    } else {
        print_success(&format!("已添加税务记录 '{}'", args.name));
    }

    Ok(())
}

trait YearExt {
    fn year(&self) -> i32;
}

impl YearExt for chrono::NaiveDate {
    fn year(&self) -> i32 {
        self.format("%Y").to_string().parse().unwrap_or(0)
    }
}
