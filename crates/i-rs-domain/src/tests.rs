#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    fn setup() {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let tmp = std::env::temp_dir()
                .join(format!("i-rs-domain-test-{}", std::process::id()));
            // ensure dir exists
            let _ = std::fs::create_dir_all(&tmp);
            unsafe { std::env::set_var("CONFIG_DIR", tmp.to_str().unwrap()); }
        });
    }

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-domain", "--help"]);
    }

    #[test]
    fn test_example() {
        setup();
        let cmd = Commands::Example {};
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, crate::presentation::OutputFormat::Table).is_ok());
    }

}