use core_error::core_errors::CoreErrors;

pub fn login_format_validation(login: String) -> Result<String, CoreErrors> {
    // Check if the length is at least 3
    if login.len() < 3 {
        return Err(CoreErrors::DataValidationError(
            "login_format_error_min_lenght".to_string(),
        ));
    }

    // Check if the login is all numeric
    if login.chars().all(|c| c.is_numeric()) {
        return Err(CoreErrors::DataValidationError(
            "login_format_error_only_numbers".to_string(),
        ));
    }

    // If both checks pass, return the valid login
    Ok(login)
}

pub fn password_format_validation(password: String) -> Result<String, CoreErrors> {
    // Check if the length is at least 6
    if password.len() < 6 {
        return Err(CoreErrors::DataValidationError(
            "password_format_error_min_lenght".to_string(),
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
        } else {
            // If there's a special character, return an error
            return Err(CoreErrors::DataValidationError(
                "password_format_error".to_string(),
            ));
        }
    }

    // Check if all conditions are met
    if has_uppercase && has_lowercase && has_digit {
        Ok(password)
    } else {
        Err(CoreErrors::DataValidationError(
            "password_format_error".to_string(),
        ))
    }
}
