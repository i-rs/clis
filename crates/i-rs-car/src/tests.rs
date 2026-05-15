#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    fn setup() {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let tmp = std::env::temp_dir()
                .join(format!("i-rs-car-test-{}", std::process::id()));
            // ensure dir exists
            let _ = std::fs::create_dir_all(&tmp);
            unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }
        });
    }

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-car", "--help"]);
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
            name: "test-i-rs-car-1".to_string(),
            license_plate: "test-license_plate".to_string(),
            brand: "test-brand".to_string(),
            model: "test-model".to_string(),
            mileage: 1.0,
            tags: None,
            remark: vec![],
        };
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Cli::try_parse_from(["i-rs-car", "list"]).unwrap().command;
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
        let cmd = Commands::Add {
            name: "test-i-rs-car-1b".to_string(),
            license_plate: "test-license_plate".to_string(),
            brand: "test-brand".to_string(),
            model: "test-model".to_string(),
            mileage: 1.0,
            tags: None,
            remark: vec![],
        };
        run(cmd, crate::presentation::OutputFormat::Table).unwrap();
        let get_cmd = Commands::Get {
            name: "test-i-rs-car-1b".to_string(),
            fuel: false,
            maintain: false,
        };
        assert!(run(get_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let update_cmd = Commands::Update {
            name: "test-i-rs-car-1b".to_string(),
            rename: None,
            license_plate: None,
            brand: None,
            model: None,
            mileage: None,
            add_tags: None,
            remove_tags: None,
            add_remark: vec![],
        };
        assert!(run(update_cmd, crate::presentation::OutputFormat::Table).is_ok());
        let del_cmd = Commands::Delete {
            name: "test-i-rs-car-1b".to_string(),
            force: true,
        };
        assert!(run(del_cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let get_cmd = Commands::Get {
            name: "nonexistent".to_string(),
            fuel: false,
            maintain: false,
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