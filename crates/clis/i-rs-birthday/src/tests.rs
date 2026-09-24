use crate::{Cli, Commands, commands, run};
use clap::Parser;

i_rs_core::test_setup!("i-rs-birthday");

#[test]
fn test_help() {
    let _ = Cli::try_parse_from(["i-rs-birthday", "--help"]);
}

#[test]
fn test_example() {
    setup();
    let cmd = Commands::Example {};
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_crud() {
    setup();
    let cmd = Commands::Add {
        name: "test-i-rs-birthday-1".to_string(),
        birth_date: "01-15".to_string(),
        year: None,
        relationship: "test-relationship".to_string(),
        tag: vec![],
        remark: vec![],
    };
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Cli::try_parse_from(["i-rs-birthday", "list"])
        .unwrap()
        .command;
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Commands::Add {
        name: "test-i-rs-birthday-1b".to_string(),
        birth_date: "01-15".to_string(),
        year: None,
        relationship: "test-relationship".to_string(),
        tag: vec![],
        remark: vec![],
    };
    run(cmd, crate::presentation::OutputFormat::Table).unwrap();
    let get_cmd = Commands::Get {
        name: "test-i-rs-birthday-1b".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
    let update_cmd = Commands::Update {
        name: "test-i-rs-birthday-1b".to_string(),
        birth_date: None,
        year: None,
        relationship: None,
        tag: None,
        remark: None,
    };
    assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
    let del_cmd = Commands::Delete {
        name: "test-i-rs-birthday-1b".to_string(),
    };
    assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_get_not_found() {
    setup();
    let get_cmd = Commands::Get {
        name: "nonexistent".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
}

#[test]
fn test_data_export() {
    setup();
    let cmd = Commands::Data(commands::data::DataCommand::Export);
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}
