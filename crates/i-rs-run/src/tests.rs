#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    fn setup() {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let tmp = std::env::temp_dir()
                .join(format!("i-rs-run-test-{}", std::process::id()));
            // ensure dir exists
            let _ = std::fs::create_dir_all(&tmp);
            unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }
        });
    }

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-run", "--help"]);
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
            date: "2024-01-15".to_string(),
            distance: 1.0,
            duration: 1.0,
            heart_rate: None,
            weather: None,
            tags: vec![],
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-run", "list"]).unwrap().command;
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

}