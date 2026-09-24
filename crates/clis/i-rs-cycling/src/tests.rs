use crate::{Cli, Commands, commands, run};
use clap::Parser;

i_rs_core::test_setup!("i-rs-cycling");

#[test]
fn test_help() {
    let _ = Cli::try_parse_from(["i-rs-cycling", "--help"]);
}

#[test]
fn test_example() {
    setup();
    let cmd = Commands::Example {};
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_add_and_list() {
    setup();
    let cmd = Commands::Add {
        date: "2024-01-15".to_string(),
        distance: 1.0,
        duration: 1,
        elevation: None,
        route: None,
        tag: vec![],
        remark: vec![],
    };
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Cli::try_parse_from(["i-rs-cycling", "list"])
        .unwrap()
        .command;
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_get_not_found() {
    setup();
    let get_cmd = Commands::Get {
        id_or_date: "00000000-0000-0000-0000-000000000000".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
}

#[test]
fn test_data_export() {
    setup();
    let cmd = Commands::Data(commands::data::DataCommand::Export);
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}
