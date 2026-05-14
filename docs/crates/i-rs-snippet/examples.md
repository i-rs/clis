# i-rs-snippet Examples

## Basic Usage

### Add a Rust Snippet

```bash
i-rs-snippet add hello \
  --language rust \
  --code 'fn main() {' \
  --code '    println!("Hello, World!");' \
  --code '}' \
  --tag rust \
  --tag hello
```

### Add a Python Snippet

```bash
i-rs-snippet add py-hello \
  --language python \
  --code 'def greet(name):' \
  --code '    print(f"Hello, {name}!")' \
  --tag python \
  --tag function
```

### Add a JavaScript Snippet

```bash
i-rs-snippet add js-async \
  --language javascript \
  --code 'async function fetchData(url) {' \
  --code '  const response = await fetch(url);' \
  --code '  return response.json();' \
  --code '}' \
  --description 'Fetch JSON data from URL' \
  --tag javascript \
  --tag async
```

## Listing and Searching

### List All Snippets

```bash
i-rs-snippet list
```

### List Snippets by Tag

```bash
i-rs-snippet list --tag rust
i-rs-snippet list --tag javascript
```

### Search Snippets

```bash
i-rs-snippet search hello
i-rs-snippet search async
i-rs-snippet search print
```

### Get Snippet Details

```bash
i-rs-snippet get hello
```

## Clipboard Operations

### Copy Snippet to Clipboard

```bash
i-rs-snippet copy hello
```

## Updating Snippets

### Update Code

```bash
i-rs-snippet update hello \
  --code 'println!("Updated version!");'
```

### Add Tags

```bash
i-rs-snippet update hello --tag useful
```

### Update Language

```bash
i-rs-snippet update old-snippet --language rust
```

## Deleting Snippets

```bash
i-rs-snippet delete hello
```

## JSON Output

### List as JSON

```bash
i-rs-snippet list --json
```

### Get as JSON

```bash
i-rs-snippet get hello --json
```

### Search as JSON

```bash
i-rs-snippet search async --json
```

## Workflow Example

```bash
# Add some snippets
i-rs-snippet add sort-rust --language rust \
  --code 'vec.sort();' \
  --tag rust \
  --tag algorithm

i-rs-snippet add sort-py --language python \
  --code 'sorted(list)' \
  --tag python \
  --tag algorithm

# Find all sorting snippets
i-rs-snippet search sort

# Copy the rust version
i-rs-snippet copy sort-rust
```
