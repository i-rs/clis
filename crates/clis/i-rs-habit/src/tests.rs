#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-habit");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-habit", "--help"]);
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
            name: "test-i-rs-habit-1".to_string(),
            description: None,
            frequency: "daily".to_string(),
            tag: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());

        let list_cmd = Commands::List {
            tag: None,
            limit: None,
            offset: None,
        };
        assert!(run(list_cmd, crate::presentation::OutputFormat::Table).is_ok());

        let cmd = Commands::Add {
            name: "test-i-rs-habit-1b".to_string(),
            description: None,
            frequency: "daily".to_string(),
            tag: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();

        let get_cmd = Commands::Get {
            id: "test-i-rs-habit-1b".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());

        let update_cmd = Commands::Update {
            id: "test-i-rs-habit-1b".to_string(),
            description: None,
            frequency: None,
            tag: None,
            remark: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());

        let del_cmd = Commands::Delete {
            id: "test-i-rs-habit-1b".to_string(),
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            id: "nonexistent".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
    }

    #[test]
    fn test_checkin() {
        setup();
        // First add a habit
        let add_cmd = Commands::Add {
            name: "test-checkin-habit".to_string(),
            description: None,
            frequency: "daily".to_string(),
            tag: vec![],
            remark: vec![],
        };
        run(add_cmd, crate::presentation::OutputFormat::Table).unwrap();

        let cmd = Commands::Checkin {
            name: "test-checkin-habit".to_string(),
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }
}
