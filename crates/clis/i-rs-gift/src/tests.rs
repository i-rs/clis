#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-gift");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-gift", "--help"]);
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
            name: "test-i-rs-gift-1".to_string(),
            gift_type: "sent".to_string(),
            recipient: "test-recipient".to_string(),
            occasion: "test-occasion".to_string(),
            value: 1.0,
            date: "2024-01-15".to_string(),
            tag: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-gift", "list"]).unwrap().command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            name: "test-i-rs-gift-1b".to_string(),
            gift_type: "sent".to_string(),
            recipient: "test-recipient".to_string(),
            occasion: "test-occasion".to_string(),
            value: 1.0,
            date: "2024-01-15".to_string(),
            tag: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            name: "test-i-rs-gift-1b".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let update_cmd = Commands::Update {
            name: "test-i-rs-gift-1b".to_string(),
            gift_type: None,
            recipient: None,
            occasion: None,
            value: None,
            date: None,
            tag: None,
            remark: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let del_cmd = Commands::Delete {
            name: "test-i-rs-gift-1b".to_string(),
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
}
