use crate::presentation::{format_table, print_entity_count, output_list};
use crate::storage;
use clap::Args;
use i_rs_core::presentation::OutputFormat;

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long, help = "标签过滤")]
    pub tag: Option<String>,
    #[arg(short, long, help = "年度过滤")]
    pub year: Option<i32>,
    #[arg(short, long, help = "税种过滤 (personal/vat)")]
    pub tax_type: Option<String>,
}

pub fn execute(args: &ListArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let store = storage::load_store()?;
    let mut entities = crate::storage::filter_by_tag(&store, args.tag.as_deref());

    if let Some(year) = args.year {
        entities.retain(|e| e.year == year);
    }

    if let Some(tax_type) = &args.tax_type {
        let tax_type_lower = tax_type.to_lowercase();
        entities.retain(|e| match tax_type_lower.as_str() {
            "personal" | "个人所得税" => matches!(e.tax_type, crate::models::TaxType::Personal),
            "vat" | "增值税" => matches!(e.tax_type, crate::models::TaxType::Vat),
            _ => true,
        });
    }

    entities.sort_by_key(|e| std::cmp::Reverse(e.date));

    if matches!(*format, OutputFormat::Json) {
        let data: Vec<_> = entities.iter().map(|e| {
            serde_json::json!({
                "name": e.name,
                "tax_type": e.tax_type,
                "amount": e.amount,
                "date": e.date.to_string(),
                "year": e.year,
                "status": e.status,
                "tags": e.tags,
                "remark": e.remark
            })
        }).collect();

        let filter = args.tag.clone().or(args.year.map(|y| y.to_string())).unwrap_or_default();
        let output = output_list(&data, entities.len(), Some(&filter), *format);
        println!("{output}");
    } else {
        if !entities.is_empty() {
            let entity_refs: Vec<_> = entities.iter().map(|e| e as &crate::models::TaxRecord).collect();
            println!("{}", format_table(&entity_refs));
        }
        print_entity_count(entities.len());
    }

    Ok(())
}
