use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum CoreErrors {
    #[error("jwt generation error : {0}")]
    JWTGenerationError(String),
    #[error("hash pasword error : {0}")]
    HashPasswordError(String),
    #[error("unknown data store error")]
    Unknown,
}

/// From LOCAL Errors into CoreErrors
///
/// JWTGenerationError
impl From<jsonwebtoken::errors::Error> for CoreErrors {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        let error_kind = err.kind();
        let error_message: String = match error_kind {
            jsonwebtoken::errors::ErrorKind::InvalidToken => "invalid_token".to_string(),
            jsonwebtoken::errors::ErrorKind::InvalidSignature => "invalid_token".to_string(),
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => "token_expired".to_string(),
            jsonwebtoken::errors::ErrorKind::Json(_) => "wrong_token_format".to_string(),
            _ => err.to_string(),
        };

        CoreErrors::JWTGenerationError(error_message)
    }
}

impl From<argon2::password_hash::Error> for CoreErrors {
    fn from(err: argon2::password_hash::Error) -> Self {
        let error_message: String = match err {
            argon2::password_hash::Error::Algorithm => "failed_protect_password".to_string(),
            argon2::password_hash::Error::B64Encoding(_) => "failed_protect_password".to_string(),
            argon2::password_hash::Error::Crypto => "failed_protect_password".to_string(),
            argon2::password_hash::Error::OutputSize {
                provided: _,
                expected: _,
            } => "failed_protect_password".to_string(),
            argon2::password_hash::Error::ParamNameDuplicated => {
                "failed_protect_password".to_string()
            }
            argon2::password_hash::Error::ParamNameInvalid => "failed_protect_password".to_string(),
            argon2::password_hash::Error::ParamValueInvalid(_) => {
                "failed_protect_password".to_string()
            }
            argon2::password_hash::Error::ParamsMaxExceeded => {
                "failed_protect_password".to_string()
            }
            argon2::password_hash::Error::Password => "invalid_password".to_string(),
            argon2::password_hash::Error::PhcStringField => "invalid_password".to_string(),
            argon2::password_hash::Error::PhcStringTrailingData => "invalid_password".to_string(),
            argon2::password_hash::Error::SaltInvalid(_) => "invalid_password".to_string(),
            argon2::password_hash::Error::Version => "invalid_password".to_string(),
            _ => err.to_string(),
        };

        CoreErrors::JWTGenerationError(error_message)
    }
}

/// From CoreError into TONIC error
impl From<CoreErrors> for Status {
    fn from(error: CoreErrors) -> Self {
        match error {
            CoreErrors::JWTGenerationError(message) => Status::invalid_argument(message),
            CoreErrors::HashPasswordError(message) => Status::permission_denied(message),
            _ => Status::internal("Internal Server Error".to_string()),
        }
    }
}
