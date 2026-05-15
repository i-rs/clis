use crate::models::{TaxRecord, TaxStatus, TaxType};
use crate::presentation::{print_success, OutputFormat};
use crate::storage;
use chrono::Utc;
use clap::Args;
use i_rs_core::presentation::output::output_item;
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
}

pub fn execute(args: &AddArgs, format: &OutputFormat) -> anyhow::Result<()> {
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

    if store.get_entry(&args.name).is_some() {
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

    store.add_entry(entry);
    storage::save_store(&store)?;

    if matches!(*format, OutputFormat::Json) {
        let data = serde_json::json!({
            "name": args.name,
            "tax_type": tax_type,
            "amount": args.amount,
            "date": date.to_string(),
            "year": year,
            "status": status,
            "tags": args.tag,
            "remark": args.remark
        });
        let output = output_item(&data, *format);
        println!("{output}");
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
