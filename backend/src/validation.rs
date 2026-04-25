use crate::error::AppError;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SQL_INJECTION_REGEX: Regex = Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|DROP|UNION|ALTER|EXEC|EXECUTE|TRUNCATE|--|\*)").unwrap();
    static ref XSS_REGEX: Regex = Regex::new(r"(?i)<script.*?>.*?</script>|on\w+?\s*=").unwrap();
}

const STELLAR_ADDRESS_LEN: usize = 56;

/// Validates a Stellar public key (G... address, 56 chars, base32).
pub fn validate_stellar_address(address: &str) -> Result<(), AppError> {
    if address.len() != STELLAR_ADDRESS_LEN
        || !address.starts_with('G')
        || !address.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Err(AppError::Validation(format!(
            "Invalid Stellar address: '{}'",
            address
        )));
    }
    Ok(())
}

/// Validates a non-empty string within a max length.
pub fn validate_string(field: &str, value: &str, max_len: usize) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{} must not be empty", field)));
    }
    if value.len() > max_len {
        return Err(AppError::Validation(format!(
            "{} must not exceed {} characters",
            field, max_len
        )));
    }
    
    // Check for suspicious SQL injection patterns
    if SQL_INJECTION_REGEX.is_match(value) {
        tracing::warn!("Suspicious SQL injection pattern detected in field {}: {}", field, value);
        return Err(AppError::Validation(format!(
            "Input contains suspicious characters or patterns in field {}",
            field
        )));
    }
    
    Ok(())
}

/// Robustly sanitizes user input to prevent XSS and other injection attacks.
pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();
    
    // Remove scripts and event handlers
    sanitized = XSS_REGEX.replace_all(&sanitized, "").to_string();
    
    // Remove HTML tags
    let mut result = String::with_capacity(sanitized.len());
    let mut in_tag = false;
    for ch in sanitized.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    // Trim and return
    result.trim().to_string()
}

/// Validates that an amount string is a positive decimal number.
pub fn validate_amount(amount: &str) -> Result<(), AppError> {
    let parsed: f64 = amount
        .parse()
        .map_err(|_| AppError::Validation("Amount must be a valid number".to_string()))?;
    if parsed <= 0.0 {
        return Err(AppError::Validation(
            "Amount must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

/// Validates an email address format.
pub fn validate_email(email: &str) -> Result<(), AppError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(email) {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }
    Ok(())
}

/// Validates a generic UUID string.
pub fn validate_uuid(uuid: &str) -> Result<(), AppError> {
    if uuid::Uuid::parse_str(uuid).is_err() {
        return Err(AppError::Validation("Invalid UUID format".to_string()));
    }
    Ok(())
}
