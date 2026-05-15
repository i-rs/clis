"""Generate update handlers + PUT routes for all i-rs-api route files.

For each route file, this script:
1. Detects the state field name and collection name
2. Adds a JSON-merge update handler
3. Registers the PUT route
"""
import os
import re

ROUTES_DIR = "crates/i-rs-api/src/routes"
SKIP = {"mod.rs", "todo.rs"}  # todo already has update

# Tools with non-'entries' BTreeMap collection names
# Format: state_field -> collection_name
CUSTOM_COLLECTIONS = {
    "budget": "budgets",
    "car": "cars",
    "debt": "debts",
    "gift": "gifts",
    "movie": "movies",
    "goal": "goals",
    "invest": "investments",
    "bookmark": "bookmarks",
    "read": "books",
    "article": "articles",
    "remind": "reminds",
    "appliance": "appliances",
    "birthday": "birthdays",
    "snippet": "snippets",
    "quote": "quotes",
    "event": "events",
    # Complex tools with non-standard BTreeMap
    "weight": "records",
}

# Tools that use service modules — need special handling
# weight uses NaiveDate as BTreeMap key, needs date parsing
SERVICE_TOOLS = {
    "habit": {"collection": "entries", "key_type": "string"},
    "keys": {"collection": "entries", "key_type": "string"},
    "kv": {"collection": "entries", "key_type": "string"},
    "mood": {"collection": "entries", "key_type": "string"},
    "note": {"collection": "entries", "key_type": "string"},
    "bookmark": {"collection": "bookmarks", "key_type": "string"},
    "weight": {"collection": "records", "key_type": "date"},
}


def detect_collection(content, state_field):
    """Detect collection name from existing delete/access patterns."""
    if state_field in CUSTOM_COLLECTIONS:
        return CUSTOM_COLLECTIONS[state_field]

    # Check delete handler pattern: store.XXX.remove(&id)
    m = re.search(rf"store\.(\w+)\.remove\(&id\)", content)
    if m:
        return m.group(1)

    # Check: store.XXX.get(&id)
    m = re.search(rf"store\.(\w+)\.get\(&id\)", content)
    if m:
        return m.group(1)

    # Check: store.XXX.values()
    m = re.search(rf"store\.(\w+)\.values\(\)", content)
    if m:
        return m.group(1)

    # Check: store.XXX.len()
    m = re.search(rf"store\.(\w+)\.len\(\)", content)
    if m:
        return m.group(1)

    return "entries"


def add_put_import(content):
    """Ensure 'put' is in the routing import."""
    if "put" in content:
        return content
    # Find: routing::{get, post, delete}
    # Replace with: routing::{get, post, put, delete}
    return re.sub(
        r'(routing::\{get, post, delete)',
        r'\1, put',
        content,
    )


def add_update_handler(content, state_field, collection):
    """Generate and insert the update handler function."""
    # Skip if handler already exists
    if f"async fn update_{state_field}(" in content:
        return content, False

    name_caps = state_field.capitalize()

    # For weight: key is a date string that maps to NaiveDate
    if state_field == "weight":
        handler = f"""
async fn update_{state_field}(
    State(state): State<Arc<AppState>>,
    Path(date): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {{
    let entry_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|_| ApiError::BadRequest(format!("Invalid date '{{date}}', expected YYYY-MM-DD")))?;
    let entry = state.{state_field}.write(|store| {{
        let entry = store.{collection}.get_mut(&entry_date)
            .ok_or_else(|| ApiError::NotFound(format!("{name_caps} '{{date}}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        entry.updated_at = chrono::Utc::now();
        Ok(entry.clone())
    }})?;
    Ok(ok_json(entry))
}}
"""
    else:
        handler = f"""
async fn update_{state_field}(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {{
    let entry = state.{state_field}.write(|store| {{
        let entry = store.{collection}.get_mut(&id)
            .ok_or_else(|| ApiError::NotFound(format!("{name_caps} '{{id}}' not found")))?;
        crate::update::merge_entry(entry, &body)
            .map_err(ApiError::BadRequest)?;
        entry.updated_at = chrono::Utc::now();
        Ok(entry.clone())
    }})?;
    Ok(ok_json(entry))
}}
"""

    # Insert before pub fn router()
    new_content = re.sub(
        r'(\n\npub fn router\(\))',
        handler + r'\n\1',
        content,
    )
    return new_content, True


def add_put_route(content, state_field):
    """Register the PUT route in the router() function."""
    # Pattern 1: .route("/{xxx}", delete(delete_xxx)) — most common
    pattern1 = rf'\.route\("/\{{\w+}}",\s*delete\(delete_{state_field}\)'
    if re.search(pattern1, content):
        return re.sub(
            rf'(\.route\("/\{{\w+}}",\s*delete\(delete_{state_field}\))\)',
            rf'\1.put(update_{state_field}))',
            content,
        )

    # Pattern 2: .route("/{xxx}", get(get_xxx)) — fallback (no delete route)
    pattern2 = rf'\.route\("/\{{\w+}}",\s*get\(get_{state_field}\)'
    if re.search(pattern2, content):
        return re.sub(
            rf'(\.route\("/\{{\w+}}",\s*get\(get_{state_field}\))\)',
            rf'\1.put(update_{state_field}))',
            content,
        )

    # Pattern 3: weight with date path param
    if state_field == "weight":
        content = re.sub(
            r'(\.route\("/\{date\}", get\(get_weight\))\)',
            r'\1.put(update_weight))',
            content,
        )

    return content


def process_file(filepath):
    filename = os.path.basename(filepath)
    if filename in SKIP:
        print(f"  SKIP {filename}: excluded")
        return False

    state_field = filename.rsplit(".", 1)[0]

    with open(filepath, "r") as f:
        content = f.read()

    # Ensure state field is used in this file
    if f"state.{state_field}." not in content:
        print(f"  SKIP {filename}: state.{state_field} not found in file")
        return False

    collection = detect_collection(content, state_field)
    original = content

    # 1. Add 'put' import if needed
    content = add_put_import(content)

    # 2. Add update handler if not exists
    content, handler_added = add_update_handler(content, state_field, collection)

    # 3. Add PUT route (even if handler already existed from partial run)
    content = add_put_route(content, state_field)

    if content == original:
        print(f"  UNCHANGED {filename}")
        return False

    with open(filepath, "w") as f:
        f.write(content)

    if handler_added:
        print(f"  OK {filename}: state={state_field}, collection={collection}")
    else:
        print(f"  ROUTE {filename}: added PUT route only")
    return True


def main():
    files = sorted(os.listdir(ROUTES_DIR))
    print(f"Processing {len(files)} route files...\n")

    updated = 0
    for file in files:
        if not file.endswith(".rs"):
            continue
        filepath = os.path.join(ROUTES_DIR, file)
        if process_file(filepath):
            updated += 1

    print(f"\nUpdated {updated} files.")


if __name__ == "__main__":
    main()
