use tonic::{metadata::MetadataMap, Status};

use super::auth_token::SessionClaims;

pub fn check_auth_token(metadata: &MetadataMap) -> Result<i64, Status> {
    match metadata.get("authorization") {
        Some(t) => {
            let token_from_header = t.to_str();
            match token_from_header {
                Ok(t) => {
                    let t = t.replace("Bearer ", "");
                    let token_data = SessionClaims::from_token(t.to_string())?;

                    Ok(token_data.sub)
                }
                Err(_) => Err(Status::unauthenticated("token_error")),
            }
        }
        _ => Err(Status::unauthenticated("token_error")),
    }
}
