#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-kv");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-kv", "--help"]);
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
            key: "test-i-rs-kv-1".to_string(),
            value: "test-value".to_string(),
            tag: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-kv", "list"]).unwrap().command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            key: "test-i-rs-kv-1b".to_string(),
            value: "test-value".to_string(),
            tag: vec![],
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            key: "test-i-rs-kv-1b".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let update_cmd = Commands::Update {
            key: "test-i-rs-kv-1b".to_string(),
            value: None,
            tag: None,
            remark: None,
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let del_cmd = Commands::Delete {
            key: "test-i-rs-kv-1b".to_string(),
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            key: "nonexistent".to_string(),
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_err());
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_search() {
        setup();
        let add_cmd = Commands::Add {
            key: "test-search-key".to_string(),
            value: "searchable-value".to_string(),
            tag: vec!["test".to_string()],
            remark: vec![],
        };
        assert!(run(add_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let search_cmd = Commands::Search {
            query: "searchable".to_string(),
        };
        assert!(run(search_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_stats() {
        setup();
        let cmd = Commands::Stats {};
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_copy_and_rename() {
        setup();
        // Add source entry
        let add_cmd = Commands::Add {
            key: "test-copy-src".to_string(),
            value: "copy-value".to_string(),
            tag: vec![],
            remark: vec![],
        };
        assert!(run(add_cmd, crate::presentation::OutputFormat::Table).is_ok());
        // Copy
        let copy_cmd = Commands::Copy {
            src: "test-copy-src".to_string(),
            dst: "test-copy-dst".to_string(),
        };
        assert!(run(copy_cmd, crate::presentation::OutputFormat::Table).is_ok());
        // Rename
        let rename_cmd = Commands::Rename {
            old: "test-copy-dst".to_string(),
            new: "test-rename-final".to_string(),
        };
        assert!(run(rename_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_list_pattern() {
        setup();
        let add_cmd = Commands::Add {
            key: "test-pattern-key".to_string(),
            value: "pattern-value".to_string(),
            tag: vec![],
            remark: vec![],
        };
        assert!(run(add_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let list_cmd = Commands::List {
            tag: None,
            pattern: Some("pattern".to_string()),
            limit: None,
            offset: None,
        };
        assert!(run(list_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }
}
