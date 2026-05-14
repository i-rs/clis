use crate::models::{TaxStatus, TaxType};
use crate::storage;
use anyhow::Result;
use chrono::{Datelike, Utc};
use clap::Parser;
use i_rs_core::presentation::print_success;

#[derive(Parser)]
pub struct UpdateArgs {
    #[arg(help = "税务记录名称")]
    pub name: String,
    #[arg(short = 'y', long, help = "新税种 (个人所得税/vat)")]
    pub tax_type: Option<String>,
    #[arg(short, long, help = "新金额")]
    pub amount: Option<f64>,
    #[arg(short, long, help = "新日期 (YYYY-MM-DD)")]
    pub date: Option<String>,
    #[arg(short, long, help = "新状态 (unreported/filing/filed/paid)")]
    pub status: Option<String>,
    #[arg(short = 'T', long, help = "新标签")]
    pub tag: Option<Vec<String>>,
    #[arg(short, long, help = "新备注")]
    pub remark: Option<Vec<String>>,
}

pub fn execute(args: &UpdateArgs) -> Result<()> {
    let mut store = storage::load_store()?;

    let record = match store.entries.get_mut(&args.name) {
        Some(r) => r,
        None => anyhow::bail!("税务记录 '{}' 不存在", args.name),
    };

    if let Some(ref t) = args.tax_type {
        record.tax_type = TaxType::from_str(t).ok_or_else(|| anyhow::anyhow!("无效税种: {}", t))?;
    }
    if let Some(a) = args.amount {
        record.amount = a;
    }
    if let Some(ref d) = args.date {
        record.date = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .map_err(|_| anyhow::anyhow!("日期格式无效，使用 YYYY-MM-DD"))?;
        record.year = record.date.year();
    }
    if let Some(ref s) = args.status {
        record.status = TaxStatus::from_str(s).ok_or_else(|| anyhow::anyhow!("无效状态: {}", s))?;
    }
    if let Some(ref t) = args.tag {
        record.tags = t.clone();
    }
    if let Some(ref r) = args.remark {
        record.remark = r.clone();
    }

    record.updated_at = Utc::now();
    storage::save_store(&store)?;

    print_success(&format!("税务记录 '{}' 更新成功", args.name));
    Ok(())
}
