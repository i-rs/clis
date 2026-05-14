mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use i_rs_core::presentation::print_error;

#[derive(Parser)]
#[command(name = "i-rs-tax")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "添加税务记录")]
    Add(commands::add::AddArgs),
    #[command(about = "列出税务记录")]
    List(commands::list::ListArgs),
    #[command(about = "查看税务记录详情")]
    Get(commands::get::GetArgs),
    #[command(about = "删除税务记录")]
    Delete(commands::delete::DeleteArgs),
    #[command(about = "查看年度统计")]
    Stats(commands::stats::StatsArgs),
    #[command(about = "显示使用示例")]
    Example(commands::example::ExampleArgs),
    #[command(about = "显示 AI 技能文档")]
    Skill(commands::skill::SkillArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Add(args) => commands::add::execute(&args),
        Commands::List(args) => commands::list::execute(&args),
        Commands::Get(args) => {
            commands::get::run(&args);
            return;
        }
        Commands::Delete(args) => {
            commands::delete::run(&args);
            return;
        }
        Commands::Stats(args) => {
            commands::stats::run(&args);
            return;
        }
        Commands::Example(args) => {
            commands::example::run(&args);
            return;
        }
        Commands::Skill(args) => {
            commands::skill::run(&args);
            return;
        }
    };

    if let Err(e) = result {
        print_error(&e.to_string());
        std::process::exit(1);
    }
}
