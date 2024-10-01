use core_error::core_errors::CoreErrors;

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
        } else {
            // If there's a special character, return an error
            return Err(CoreErrors::DataValidationError(
                "password_format_error_invalid_character".to_string(),
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

/// Validates the format of the email string.
///
/// Checks:
/// - Contains exactly one '@' symbol.
/// - Non-empty local and domain parts.
/// - Domain contains at least one '.' character.
///
/// # Arguments
///
/// * `email` - A `String` representing the user's email.
///
/// # Returns
///
/// * `Ok(String)` if validation passes.
/// * `Err(CoreErrors)` if validation fails.
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
