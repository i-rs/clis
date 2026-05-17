#[cfg(test)]
mod tests {
    use crate::presentation::OutputFormat;
    use crate::{Commands, commands, run};
    use clap::Parser;

    i_rs_core::test_setup!("i-rs-goal");

    #[test]
    fn test_help() {
        let _ = crate::Cli::try_parse_from(["i-rs-goal", "--help"]);
    }

    #[test]
    fn test_example() {
        setup();
        let cmd = Commands::Example(commands::example::ExampleArgs {});
        assert!(run(cmd, OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_crud() {
        setup();

        // Add
        let cmd = Commands::Add(commands::add::AddArgs {
            name: "test-goal-1".to_string(),
            target: 1000.0,
            deadline: "2027-12-31".to_string(),
            tags: vec![],
            remark: vec![],
            milestones: vec![],
        });
        assert!(run(cmd, OutputFormat::Table).is_ok());

        // List
        let cmd = Commands::List(commands::list::ListArgs { tag: None });
        assert!(run(cmd, OutputFormat::Table).is_ok());

        // Add second
        let cmd = Commands::Add(commands::add::AddArgs {
            name: "test-goal-1b".to_string(),
            target: 500.0,
            deadline: "2027-06-30".to_string(),
            tags: vec![],
            remark: vec![],
            milestones: vec![],
        });
        assert!(run(cmd, OutputFormat::Table).is_ok());

        // Get
        let cmd = Commands::Get(commands::get::GetArgs {
            name: "test-goal-1b".to_string(),
        });
        assert!(run(cmd, OutputFormat::Table).is_ok());

        // Update
        let cmd = Commands::Update(commands::update::UpdateArgs {
            name: "test-goal-1b".to_string(),
            target: Some(600.0),
            deadline: None,
            tags: None,
            remark: None,
        });
        assert!(run(cmd, OutputFormat::Table).is_ok());

        // Delete
        let cmd = Commands::Delete(commands::delete::DeleteArgs {
            name: "test-goal-1b".to_string(),
        });
        assert!(run(cmd, OutputFormat::Table).is_ok());
    }

    #[test]
    fn test_get_not_found() {
        setup();
        let cmd = Commands::Get(commands::get::GetArgs {
            name: "nonexistent".to_string(),
        });
        assert!(run(cmd, OutputFormat::Table).is_err());
    }

    #[test]
    fn test_data_export() {
        setup();
        let cmd = Commands::Data(commands::data::DataCommand::Export);
        assert!(run(cmd, OutputFormat::Table).is_ok());
    }
}
