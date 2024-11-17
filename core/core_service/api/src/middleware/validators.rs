use core_error::core_errors::CoreErrors;
use regex::Regex;

/// Validates the format of the login string.
///
/// Checks:
/// - Minimum length of 3 characters.
/// - Not entirely numeric.
/// - Does not contain any whitespace characters.
///
/// # Arguments
///
/// * `login` - A `String` representing the user's login.
///
/// # Returns
///
/// * `Ok(String)` if validation passes.
/// * `Err(CoreErrors)` if validation fails.
pub fn login_format_validation(login: String) -> Result<String, CoreErrors> {
    // Check if the length is at least 3
    if login.len() < 3 {
        return Err(CoreErrors::DataValidationError(
            "login_format_error_min_length".to_string(),
        ));
    }

    // Check if the login is all numeric
    if login.chars().all(|c| c.is_numeric()) {
        return Err(CoreErrors::DataValidationError(
            "login_format_error_only_numbers".to_string(),
        ));
    }

    // Check if the login contains any whitespace characters
    if login.chars().any(|c| c.is_whitespace()) {
        return Err(CoreErrors::DataValidationError(
            "login_format_error_no_whitespace".to_string(),
        ));
    }

    // If all checks pass, return the valid login
    Ok(login)
}

/// Validates the format of the password string.
///
/// Checks:
/// - Minimum length of 6 characters.
/// - Contains at least one uppercase letter.
/// - Contains at least one lowercase letter.
/// - Contains at least one digit.
/// - Does not contain any whitespace characters.
/// - Contains only allowed characters (no special characters).
///
/// # Arguments
///
/// * `password` - A `String` representing the user's password.
///
/// # Returns
///
/// * `Ok(String)` if validation passes.
/// * `Err(CoreErrors)` if validation fails.
pub fn password_format_validation(password: String) -> Result<String, CoreErrors> {
    // Check if the length is at least 6
    if password.len() < 6 {
        return Err(CoreErrors::DataValidationError(
            "password_format_error_min_length".to_string(),
        ));
    }

    // Flags for validation
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_digit = false;

    // Iterate through each character in the password
    for c in password.chars() {
        if c.is_uppercase() {
            has_uppercase = true;
        } else if c.is_lowercase() {
            has_lowercase = true;
        } else if c.is_numeric() {
            has_digit = true;
        } else if c.is_whitespace() {
            // If there's a whitespace character, return an error
            return Err(CoreErrors::DataValidationError(
                "password_format_error_no_whitespace".to_string(),
            ));
        }
    }

    // Check if all required conditions are met
    if has_uppercase && has_lowercase && has_digit {
        Ok(password)
    } else {
        Err(CoreErrors::DataValidationError(
            "password_format_error_missing_requirements".to_string(),
        ))
    }
}

pub fn email_format_validation(email: String) -> Result<String, CoreErrors> {
    // Split the email into local and domain parts
    let parts: Vec<&str> = email.split('@').collect();

    // There must be exactly one '@' symbol
    if parts.len() != 2 {
        return Err(CoreErrors::DataValidationError(
            "email_format_error_invalid_at_symbol".to_string(),
        ));
    }

    let local = parts[0];
    let domain = parts[1];

    // Local and domain parts must not be empty
    if local.is_empty() || domain.is_empty() {
        return Err(CoreErrors::DataValidationError(
            "email_format_error_empty_local_or_domain".to_string(),
        ));
    }

    // Domain must contain at least one '.' character
    if !domain.contains('.') {
        return Err(CoreErrors::DataValidationError(
            "email_format_error_invalid_domain".to_string(),
        ));
    }

    // Optionally, you can add more checks here (e.g., valid characters, length restrictions)

    Ok(email)
}

/// A composite validator that accepts multiple validation functions.
pub struct CompositValidator {
    validators: Vec<fn(String) -> Result<String, CoreErrors>>,
}

impl CompositValidator {
    /// Creates a new `CompositValidator` with a set of validators.
    pub fn new(validators: Vec<fn(String) -> Result<String, CoreErrors>>) -> Self {
        CompositValidator { validators }
    }

    /// Applies all validators in sequence to the given value.
    pub fn validate(&self, value: String) -> Result<String, CoreErrors> {
        self.validators
            .iter()
            .try_fold(value, |acc, validate_fn| validate_fn(acc))
    }
}

/// Validator that checks if the value is not empty.
pub fn empty_validation(value: String) -> Result<String, CoreErrors> {
    if value.is_empty() {
        return Err(CoreErrors::DataValidationError(
            "validator_empty".to_string(),
        ));
    }
    Ok(value)
}

/// Validator that checks if the value has at least 3 symbols.
pub fn min_symbols_validator_3(value: String) -> Result<String, CoreErrors> {
    if value.len() < 3 {
        return Err(CoreErrors::DataValidationError(
            "validator_min_symbols".to_string(),
        ));
    }
    Ok(value)
}

/// Validator that checks if the value does not exceed 20 symbols.
pub fn max_symbols_validator_20(value: String) -> Result<String, CoreErrors> {
    if value.len() > 20 {
        return Err(CoreErrors::DataValidationError(
            "validator_max_symbols".to_string(),
        ));
    }
    Ok(value)
}

/// Validator that checks if the value does not exceed 250 symbols.
pub fn max_symbols_validator_250(value: String) -> Result<String, CoreErrors> {
    if value.len() > 250 {
        return Err(CoreErrors::DataValidationError(
            "validator_max_symbols".to_string(),
        ));
    }
    Ok(value)
}

/// Validator that checks if the value contains only alphanumeric characters and spaces.
pub fn no_special_symbols_validator(value: String) -> Result<String, CoreErrors> {
    if value.chars().any(|c| !c.is_alphanumeric() && c != ' ') {
        return Err(CoreErrors::DataValidationError(
            "validator_special_symbols".to_string(),
        ));
    }
    Ok(value)
}

// Validator that checks if the value is a valid hexadecimal color using a regular expression.
///
/// A valid hexadecimal color starts with a '#' followed by exactly 3 or 6 valid hexadecimal characters.
///
/// # Arguments
///
/// * `value` - A `String` representing the color.
///
/// # Returns
///
/// * `Ok(String)` if the value is a valid hex color.
/// * `Err(CoreErrors)` if the value is not a valid hex color.
pub fn hex_color_validator(value: String) -> Result<String, CoreErrors> {
    // Define the regex pattern for a hex color
    let hex_color_regex = Regex::new(r"^#([A-Fa-f0-9]{3}|[A-Fa-f0-9]{6})$").unwrap();

    // Check if the value matches the regex
    if hex_color_regex.is_match(&value) {
        Ok(value)
    } else {
        Err(CoreErrors::DataValidationError(
            "validator_invalid_hex_color".to_string(),
        ))
    }
}
