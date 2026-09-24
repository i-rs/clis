use crate::{Cli, Commands, commands, run};
use clap::Parser;

i_rs_core::test_setup!("i-rs-podcast");

#[test]
fn test_help() {
    let _ = Cli::try_parse_from(["i-rs-podcast", "--help"]);
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
        name: "test-i-rs-podcast-1".to_string(),
        author: None,
        duration: None,
        tag: vec![],
        remark: vec![],
        notes: vec![],
    };
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Cli::try_parse_from(["i-rs-podcast", "list"])
        .unwrap()
        .command;
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    let cmd = Commands::Add {
        name: "test-i-rs-podcast-1b".to_string(),
        author: None,
        duration: None,
        tag: vec![],
        remark: vec![],
        notes: vec![],
    };
    run(cmd, crate::presentation::OutputFormat::Table).unwrap();
    let get_cmd = Commands::Get {
        name: "test-i-rs-podcast-1b".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
    let update_cmd = Commands::Update {
        name: "test-i-rs-podcast-1b".to_string(),
        author: None,
        duration: None,
        tag: None,
        remark: None,
        notes: None,
    };
    assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
    let del_cmd = Commands::Delete {
        name: "test-i-rs-podcast-1b".to_string(),
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
