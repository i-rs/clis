#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Name cannot be empty".to_string(),
        });
    }

    if name.len() > 100 {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Name cannot exceed 100 characters".to_string(),
        });
    }

    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(ValidationError {
            field: "name".to_string(),
            message: format!(
                "Name contains invalid character. Avoid: {}",
                invalid_chars.iter().collect::<String>()
            ),
        });
    }

    Ok(())
}

pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError {
            field: "url".to_string(),
            message: "URL cannot be empty".to_string(),
        });
    }

    let lower = url.to_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err(ValidationError {
            field: "url".to_string(),
            message: "URL must start with http:// or https://".to_string(),
        });
    }

    if url.len() > 2000 {
        return Err(ValidationError {
            field: "url".to_string(),
            message: "URL is too long (max 2000 characters)".to_string(),
        });
    }

    Ok(())
}

pub fn validate_weight(weight: f64) -> Result<(), ValidationError> {
    if weight <= 0.0 {
        return Err(ValidationError {
            field: "weight".to_string(),
            message: "Weight must be greater than 0".to_string(),
        });
    }

    if weight > 1000.0 {
        return Err(ValidationError {
            field: "weight".to_string(),
            message: "Weight seems unrealistic (max 1000 kg)".to_string(),
        });
    }

    Ok(())
}

pub fn validate_amount(amount: f64) -> Result<(), ValidationError> {
    if amount <= 0.0 {
        return Err(ValidationError {
            field: "amount".to_string(),
            message: "Amount must be greater than 0".to_string(),
        });
    }

    if amount > 1_000_000_000.0 {
        return Err(ValidationError {
            field: "amount".to_string(),
            message: "Amount seems unrealistic (max 1 billion)".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("hello").is_ok());
        assert!(validate_name(&"a".repeat(100)).is_ok());
        assert!(validate_name("my-entry_123").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let err = validate_name("").unwrap_err();
        assert_eq!(err.field, "name");
    }

    #[test]
    fn test_validate_name_too_long() {
        let err = validate_name(&"a".repeat(101)).unwrap_err();
        assert_eq!(err.field, "name");
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        assert!(validate_name("hello/world").is_err());
        assert!(validate_name("path\\name").is_err());
        assert!(validate_name("name:sub").is_err());
        assert!(validate_name("name*star").is_err());
        assert!(validate_name("name?quest").is_err());
        assert!(validate_name("<tag>").is_err());
        assert!(validate_name("pipe|").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("https://example.com").is_ok());
        // "https://a" = 9 chars, 200 times = 1800 chars (under 2000 limit)
        assert!(validate_url(&"https://a".repeat(200)).is_ok());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_validate_url_no_scheme() {
        assert!(validate_url("example.com").is_err());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_validate_url_too_long() {
        let long = "https://a".repeat(1001);
        assert!(validate_url(&long).is_err());
    }

    #[test]
    fn test_validate_weight_valid() {
        assert!(validate_weight(1.0).is_ok());
        assert!(validate_weight(500.0).is_ok());
        assert!(validate_weight(1000.0).is_ok());
    }

    #[test]
    fn test_validate_weight_zero() {
        assert!(validate_weight(0.0).is_err());
        assert!(validate_weight(-1.0).is_err());
    }

    #[test]
    fn test_validate_weight_too_high() {
        assert!(validate_weight(1000.1).is_err());
    }

    #[test]
    fn test_validate_amount_valid() {
        assert!(validate_amount(1.0).is_ok());
        assert!(validate_amount(1_000_000_000.0).is_ok());
    }

    #[test]
    fn test_validate_amount_zero() {
        assert!(validate_amount(0.0).is_err());
        assert!(validate_amount(-1.0).is_err());
    }

    #[test]
    fn test_validate_amount_too_high() {
        assert!(validate_amount(1_000_000_001.0).is_err());
    }
}
