use core_error::core_errors::CoreErrors;
use lazy_static::lazy_static;
use std::env;
use std::time::SystemTime;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const SESSION_TOKEN_EXP: usize = 86400000; // 24 hours in ms
const REFRESH_TOKEN_EXP: usize = 2592000000; // 30 days in ms

lazy_static! {
    static ref JWT_SECRET: String =
        env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY environment variable is not set");
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionClaims {
    // The sub depicts the so-called subject, so “who,” in this case (user ID)
    pub sub: i64,
    // Date when token expires
    exp: usize,
}

impl SessionClaims {
    pub fn new(sub: i64) -> SessionClaims {
        let now: usize = get_now_time_in_ms();

        SessionClaims {
            sub,
            exp: now + SESSION_TOKEN_EXP,
        }
    }

    pub fn into_token(&self) -> Result<String, CoreErrors> {
        let token = encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )?;

        Ok(token)
    }
}

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    // The sub depicts the so-called subject, so “who,” in this case (user ID)
    pub sub: i64,
    // Date when token expires
    exp: usize,

    is_refresh: bool,
}

impl RefreshClaims {
    pub fn new(sub: i64) -> RefreshClaims {
        let now: usize = get_now_time_in_ms();

        RefreshClaims {
            exp: now + REFRESH_TOKEN_EXP,
            sub,
            is_refresh: true,
        }
    }

    pub fn from_token(token: String) -> Result<RefreshClaims, CoreErrors> {
        let token = decode::<RefreshClaims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &Validation::default(),
        )?;

        Ok(token.claims)
    }

    pub fn into_token(&self) -> Result<String, CoreErrors> {
        let token = encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(JWT_SECRET.as_ref()),
        )?;

        return Ok(token);
    }
}

fn get_now_time_in_ms() -> usize {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize
}
