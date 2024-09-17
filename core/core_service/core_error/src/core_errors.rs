use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum CoreErrors {
    #[error("jwt generation error : {0}")]
    JWTGenerationError(String),
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

/// From CoreError into TONIC error
impl From<CoreErrors> for Status {
    fn from(error: CoreErrors) -> Self {
        match error {
            CoreErrors::JWTGenerationError(message) => Status::invalid_argument(message),
            _ => Status::internal("Internal Server Error".to_string()),
        }
    }
}
