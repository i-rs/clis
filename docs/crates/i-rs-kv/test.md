# i-rs-kv Test Records

## Test Data

```bash
# Simple values
i-rs-kv add "name" --value "John"
i-rs-kv add "email" --value "john@example.com"
i-rs-kv add "theme" --value "dark"

# URLs
i-rs-kv add "github" --value "https://github.com/john"
i-rs-kv add "api" --value "https://api.example.com"

# With tags
i-rs-kv add "work-email" --value "john@company.com" --tag work
i-rs-kv add "personal-email" --value "john@gmail.com" --tag personal
```