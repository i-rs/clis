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