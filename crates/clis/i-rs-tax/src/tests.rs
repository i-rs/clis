#[cfg(test)]
mod tests {
    use crate::{Cli, Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-tax");

    #[test]
    fn test_help() {
        let _ = Cli::try_parse_from(["i-rs-tax", "--help"]);
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, i_rs_core::presentation::OutputFormat::Table).is_ok());
    }
}
