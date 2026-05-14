#![allow(dead_code)]
mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};

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
    #[command(about = "更新税务记录")]
    Update(commands::update::UpdateArgs),
    #[command(about = "查看年度统计")]
    Stats(commands::stats::StatsArgs),
    #[command(about = "显示使用示例")]
    Example(commands::example::ExampleArgs),
    #[command(about = "显示 AI 技能文档")]
    Skill {
        #[arg(value_name = "SUB_COMMAND")]
        sub: Option<String>,
    },
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
        Commands::Update(ref args) => {
            commands::update::execute(args)
        }
        Commands::Stats(args) => {
            commands::stats::run(&args);
            return;
        }
        Commands::Example(args) => {
            commands::example::run(&args);
            return;
        }
        Commands::Skill { sub } => {
            commands::skill::handle_skill(sub.map(|s| match s.as_str() {
                "summary" => commands::skill::SkillCommand::Summary,
                "content" => commands::skill::SkillCommand::Content,
                _ => commands::skill::SkillCommand::Raw,
            }));
            Ok(())
        }
    };

    i_rs_core::exit_on_error!(result, false);
}
