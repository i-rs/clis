#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-mood");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-mood", "--help"]);
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
            date: "2024-01-15".to_string(),
            mood: "test-mood".to_string(),
            tag: vec![],
            content: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-mood", "list"]).unwrap().command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            date: "2024-01-16".to_string(),
            mood: "test-mood".to_string(),
            tag: vec![],
            content: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            date: "2024-01-16".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let update_cmd = Commands::Update {
            date: "2024-01-16".to_string(),
            mood: None,
            tag: None,
            content: None,
            remark: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let del_cmd = Commands::Delete {
            date: "2024-01-16".to_string(),
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            date: "nonexistent".to_string(),
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
