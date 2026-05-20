#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-meal");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-meal", "--help"]);
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
            meal_type: "breakfast".to_string(),
            food_items: "test-food".to_string(),
            date: Some("2024-01-15".to_string()),
            calories: None,
            tag: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-meal", "list"]).unwrap().command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            meal_type: "lunch".to_string(),
            food_items: "test-food-2".to_string(),
            date: Some("2024-01-16".to_string()),
            calories: None,
            tag: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            date: None,
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
        let update_cmd = Commands::Update {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
            meal_type: None,
            food_items: None,
            calories: None,
            tag: None,
            remark: None,
            date: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_err());
        let del_cmd = Commands::Delete {
            id: "00000000-0000-0000-0000-000000000000".to_string(),
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_err());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            id: "nonexistent".to_string(),
            date: None,
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
