use serde::de::DeserializeOwned;
use serde::Serialize;

/// Merge a JSON body into an existing entity.
/// Each field in `body` overwrites the corresponding field in `entry`.
/// Fields not present in `body` retain their current values.
/// If `entry` has an `updated_at` field, it is set to the current UTC time.
pub fn merge_entry<T: Serialize + DeserializeOwned>(
    entry: &mut T,
    body: &serde_json::Value,
) -> Result<(), String> {
    let mut current =
        serde_json::to_value(&*entry).map_err(|e| format!("Serialize error: {e}"))?;
    if let (Some(obj), Some(body_obj)) = (current.as_object_mut(), body.as_object()) {
        for (k, v) in body_obj {
            // Skip id/key fields — can't be changed via update
            if k == "name" || k == "id" {
                continue;
            }
            obj.insert(k.clone(), v.clone());
        }
    }
    // Auto-set updated_at if the entity has this field
    if let Some(obj) = current.as_object() {
        if obj.contains_key("updated_at") {
            let obj = current.as_object_mut().unwrap();
            obj.insert(
                "updated_at".to_string(),
                serde_json::json!(chrono::Utc::now()),
            );
        }
    }
    *entry = serde_json::from_value(current).map_err(|e| format!("Invalid field values: {e}"))?;
    Ok(())
}
