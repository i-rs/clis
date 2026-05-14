# i-rs-snippet Test Records

## Test Setup

Create test snippets for verification.

```bash
# Add test snippet 1
i-rs-snippet add test-hello \
  --language rust \
  --code 'fn main() {' \
  --code '    println!("Test Hello");' \
  --code '}' \
  --tag test \
  --tag rust

# Add test snippet 2
i-rs-snippet add test-python \
  --language python \
  --code 'print("Test Python")' \
  --tag test \
  --tag python

# Add test snippet 3
i-rs-snippet add test-js \
  --language javascript \
  --code 'console.log("Test JS");' \
  --description 'Test JavaScript snippet' \
  --tag test \
  --tag javascript
```

## Test Commands

```bash
# List all
i-rs-snippet list

# List by tag
i-rs-snippet list --tag test

# Search
i-rs-snippet search test
i-rs-snippet search hello
i-rs-snippet search python

# Get details
i-rs-snippet get test-hello
i-rs-snippet get test-python

# Update
i-rs-snippet update test-hello --tag updated
i-rs-snippet get test-hello

# Delete
i-rs-snippet delete test-hello
i-rs-snippet delete test-python
i-rs-snippet delete test-js

# Verify deletion
i-rs-snippet list
```

## Expected Results

- List shows all added snippets with name, language, tags, timestamps
- Search finds snippets by name, language, code content, or tags
- Get shows full snippet details including code
- Update modifies snippet and updates timestamp
- Delete removes snippet
- Empty list after cleanup
