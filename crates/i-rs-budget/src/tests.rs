#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-budget");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-budget", "--help"]);
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
            category: "test-i-rs-budget-1".to_string(),
            amount: 1.0,
            period: None,
            tags: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-budget", "list"])
            .unwrap()
            .command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            category: "test-i-rs-budget-1b".to_string(),
            amount: 1.0,
            period: None,
            tags: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            category: Some("test-i-rs-budget-1b".to_string()),
            expense_id: None,
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let update_cmd = Commands::Update {
            category: "test-i-rs-budget-1b".to_string(),
            amount: None,
            period: None,
            tags: None,
            remark: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let del_cmd = Commands::Delete {
            category: Some("test-i-rs-budget-1b".to_string()),
            expense_id: None,
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            category: Some("nonexistent".to_string()),
            expense_id: None,
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }
}
