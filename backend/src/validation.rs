use crate::error::AppError;

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
    Ok(())
}

/// Strips HTML tags and trims whitespace from user input.
pub fn sanitize_input(input: &str) -> String {
    // Remove anything that looks like an HTML/script tag
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
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
