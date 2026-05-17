use axum::Json;

/// Wrap any serializable value into a success JSON response.
pub fn ok_json<T: serde::Serialize>(data: T) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true, "data": data }))
}

/// Wrap a list into a success JSON response with count metadata.
pub fn ok_json_list<T: serde::Serialize>(data: Vec<T>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "data": data,
        "meta": { "count": data.len() }
    }))
}

/// Wrap a unit success response.
pub fn ok_json_message() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "success": true, "message": "OK" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_json_contains_success_and_data() {
        let response = ok_json("hello");
        let v = response.0;
        assert_eq!(v["success"], true);
        assert_eq!(v["data"], "hello");
    }

    #[test]
    fn test_ok_json_with_number() {
        let response = ok_json(42);
        assert_eq!(response.0["data"], 42);
    }

    #[test]
    fn test_ok_json_list_with_count() {
        let response = ok_json_list(vec![1, 2, 3]);
        let v = response.0;
        assert_eq!(v["success"], true);
        assert_eq!(v["meta"]["count"], 3);
        assert_eq!(v["data"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_ok_json_list_empty() {
        let response = ok_json_list::<i32>(vec![]);
        assert_eq!(response.0["meta"]["count"], 0);
        assert!(response.0["data"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_ok_json_message() {
        let response = ok_json_message();
        assert_eq!(response.0["success"], true);
        assert_eq!(response.0["message"], "OK");
    }
}
