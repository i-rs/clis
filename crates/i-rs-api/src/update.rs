use serde::de::DeserializeOwned;
use serde::Serialize;

/// Merge a JSON body into an existing entity (partial update).
///
/// Each field in `body` overwrites the corresponding field in `entry`.
/// Fields not present in `body` retain their current values.
/// Set a field to `null` to remove it (resets to its default value).
/// If `entry` has an `updated_at` field, it is set to the current UTC time.
pub fn merge_entry<T: Serialize + DeserializeOwned>(
    entry: &mut T,
    body: &serde_json::Value,
) -> Result<(), String> {
    let mut current =
        serde_json::to_value(&*entry).map_err(|e| format!("Serialize error: {e}"))?;
    if let (Some(obj), Some(body_obj)) = (current.as_object_mut(), body.as_object()) {
        for (k, v) in body_obj {
            // Skip readonly key fields
            if k == "name" || k == "id" {
                continue;
            }
            if v.is_null() {
                // Remove field to reset to default
                obj.remove(k);
            } else if let (Some(existing), Some(body_obj)) =
                (obj.get(k).and_then(|o| o.as_object()), v.as_object())
            {
                // Deep merge for nested objects
                let mut merged = existing.clone();
                for (nk, nv) in body_obj {
                    merged.insert(nk.clone(), nv.clone());
                }
                obj.insert(k.clone(), serde_json::Value::Object(merged));
            } else {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    // Auto-set updated_at if the entity has this field
    if let Some(obj) = current.as_object()
        && obj.contains_key("updated_at")
    {
        let obj = current.as_object_mut().unwrap();
        obj.insert(
            "updated_at".to_string(),
            serde_json::json!(chrono::Utc::now().timestamp()),
        );
    }
    *entry = serde_json::from_value(current).map_err(|e| format!("Invalid field values: {e}"))?;
    Ok(())
}
