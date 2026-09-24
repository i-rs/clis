use crate::{Cli, Commands, commands, run};
use clap::Parser;

i_rs_core::test_setup!("i-rs-sleep");

#[test]
fn test_help() {
    let _ = Cli::try_parse_from(["i-rs-sleep", "--help"]);
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
        bedtime: "22:00".to_string(),
        wake_time: "22:00".to_string(),
        quality: 1,
        tag: vec![],
        remark: vec![],
    };
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Cli::try_parse_from(["i-rs-sleep", "list"]).unwrap().command;
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_get_not_found() {
    setup();
    let get_cmd = Commands::Get {
        id: "00000000-0000-0000-0000-000000000000".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
}

#[test]
fn test_data_export() {
    setup();
    let cmd = Commands::Data(commands::data::DataCommand::Export);
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}
