use clap::Parser;

#[derive(Parser, Debug)]
pub struct ExampleArgs {}

pub fn example(_args: ExampleArgs) -> anyhow::Result<()> {
    println!(
        r#"
i-rs-goal Examples:

1. Create a savings goal:
   i-rs-goal add "Emergency Fund" --target 10000 --deadline 2025-12-31 --tags emergency,finance

2. List all goals:
   i-rs-goal list

3. List goals with specific tag:
   i-rs-goal list --tag emergency

4. View goal details:
   i-rs-goal get "Emergency Fund"

5. Deposit to a goal:
   i-rs-goal deposit "Emergency Fund" --amount 500

6. Add a milestone:
   i-rs-goal milestone -g "Emergency Fund" -n "First 1000" -a 1000

7. List milestones:
   i-rs-goal milestone -g "Emergency Fund" --list

8. Remove a milestone:
   i-rs-goal milestone -g "Emergency Fund" -r <milestone-id>

9. Update goal:
   i-rs-goal update "Emergency Fund" --target 15000

10. View statistics:
    i-rs-goal stats

11. Delete a goal:
    i-rs-goal delete "Old Goal"

12. JSON output (for scripting):
    i-rs-goal list --json
    i-rs-goal get "Emergency Fund" --json
"#
    );

    Ok(())
}
