use crate::presentation::{output_item, print_success};
use crate::storage;
use clap::Args;
use i_rs_core::presentation::OutputFormat;

#[derive(Args)]
pub struct DeleteArgs {
    #[arg(help = "税务记录名称")]
    pub name: String,
}

pub fn execute(args: &DeleteArgs, format: &OutputFormat) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    if store.get_entry(&args.name).is_none() {
        anyhow::bail!("税务记录 '{}' 不存在", args.name);
    }

    store.remove_entry(&args.name);
    storage::save_store(&store)?;

    if matches!(*format, OutputFormat::Json) {
        let data = serde_json::json!({
            "deleted": args.name
        });
        let output = output_item(&data, *format);
        println!("{output}");
    } else {
        print_success(&format!("已删除税务记录 '{}'", args.name));
    }

    Ok(())
}
