use crate::{Cli, Commands, commands, run};
use clap::Parser;

i_rs_core::test_setup!("i-rs-weight");

#[test]
fn test_help() {
    let _ = Cli::try_parse_from(["i-rs-weight", "--help"]);
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

    // Use service to create a record and capture its ID
    let mut store = crate::storage::load_store().unwrap();
    let record = crate::service::add_weight(
        &mut store,
        Some("2024-01-15".to_string()),
        1.0,
        vec![],
        vec![],
    )
    .unwrap();
    let id1 = record.id.clone();

    let record2 = crate::service::add_weight(
        &mut store,
        Some("2024-01-16".to_string()),
        1.0,
        vec![],
        vec![],
    )
    .unwrap();
    let id2 = record2.id.clone();
    crate::storage::save_store(&store).unwrap();

    // List
    let cmd = Commands::List {
        days: None,
        chart: false,
        stats: false,
    };
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());

    // Get by id (short prefix)
    let get_cmd = Commands::Get {
        id: id2[..8].to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());

    // Update by id
    let update_cmd = Commands::Update {
        id: id2[..8].to_string(),
        weight: None,
        tag: None,
        remark: None,
    };
    assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());

    // Delete by id
    let del_cmd = Commands::Delete {
        id: id1[..8].to_string(),
    };
    assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
}

#[test]
fn test_get_not_found() {
    setup();
    let get_cmd = Commands::Get {
        id: "nonexist".to_string(),
    };
    assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
}

#[test]
fn test_data_export() {
    setup();
    let cmd = Commands::Data(commands::data::DataCommand::Export);
    assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
}
