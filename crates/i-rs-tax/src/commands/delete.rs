use crate::presentation::print_success;
use crate::storage;
use clap::Args;

#[derive(Args)]
pub struct DeleteArgs {
    #[arg(help = "税务记录名称")]
    pub name: String,
    #[arg(long, default_value = "false")]
    pub json: bool,
}

pub fn execute(args: &DeleteArgs) -> anyhow::Result<()> {
    let mut store = storage::load_store()?;

    if !store.entries.contains_key(&args.name) {
        anyhow::bail!("税务记录 '{}' 不存在", args.name);
    }

    storage::remove_entry(&mut store, &args.name);
    storage::save_store(&store)?;

    if args.json {
        println!("{}", serde_json::json!({
            "success": true,
            "data": {
                "deleted": args.name
            }
        }));
    } else {
        print_success(&format!("已删除税务记录 '{}'", args.name));
    }

    Ok(())
}

pub fn run(args: &DeleteArgs) {
    if let Err(_e) = execute(args) {
        std::process::exit(1);
    }
}
