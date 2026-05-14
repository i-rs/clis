# i-rs-tick Test Records

## Test Data

```bash
# Meetings
i-rs-tick add "Standup" --duration 900 --tag daily --tag meeting
i-rs-tick add "Sprint Planning" --duration 3600 --tag meeting --tag project
i-rs-tick add "Code Review" --duration 1800 --tag collaboration

# Development
i-rs-tick add "Feature Development" --duration 14400 --tag coding --tag project
i-rs-tick add "Bug Fix" --duration 3600 --tag coding --tag bug
i-rs-tick add "Testing" --duration 7200 --tag testing --tag project

# Documentation
i-rs-tick add "Write Docs" --duration 3600 --tag documentation
i-rs-tick add "Update README" --duration 1800 --tag documentation
```