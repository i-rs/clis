mod commands;
mod models;
mod presentation;
mod storage;

use clap::{Parser, Subcommand};
use commands::{handle_skill};
use i_rs_core::presentation::OutputFormat;

#[derive(Parser)]
#[command(name = "i-rs-tax")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    json: bool,
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
    #[clap(subcommand)]
    Skill(commands::skill::SkillCommand),
    #[clap(subcommand)]
    Data(commands::data::DataCommand),
}

fn main() {
    let cli = Cli::parse();
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Table
    };

    i_rs_core::exit_on_error!(run(cli.command, format), cli.json);
}

fn run(command: Commands, format: OutputFormat) -> anyhow::Result<()> {
    match command {
        Commands::Add(args) => commands::add::execute(&args, &format),
        Commands::List(args) => commands::list::execute(&args, &format),
        Commands::Get(args) => commands::get::execute(&args, &format),
        Commands::Delete(args) => commands::delete::execute(&args, &format),
        Commands::Update(args) => commands::update::execute(&args),
        Commands::Stats(args) => commands::stats::execute(&args, &format),
        Commands::Example(args) => commands::example::execute(&args, &format),
        Commands::Skill(cmd) => {
            handle_skill(&cmd)?;
            Ok(())
        }
        Commands::Data(commands) => commands::data::handle(&commands),
    }
}

#[cfg(test)]
mod tests;
