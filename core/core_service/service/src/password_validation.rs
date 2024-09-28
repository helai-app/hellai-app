use std::env;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use core_error::core_errors::CoreErrors;
use lazy_static::lazy_static;

lazy_static! {
    static ref JWT_SECRET: String = env::var("PASSWORD_SECRET_KEY")
        .expect("PASSWORD_SECRET_KEY environment variable is not set");
}

pub fn hash_password(password: &str) -> Result<(String, String), CoreErrors> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::new_with_secret(
        JWT_SECRET.as_bytes(),
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::DEFAULT,
    )
    .unwrap();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok((password_hash, salt.as_str().to_string()))
}

pub fn verify_hash_password(hash: &str, password: &str) -> Result<bool, CoreErrors> {
    let parsed_hash = PasswordHash::new(hash)?;

    println!("parsed_hash: {}", parsed_hash);

    let argon2 = Argon2::new_with_secret(
        JWT_SECRET.as_bytes(),
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::DEFAULT,
    )
    .unwrap();

    let is_valide_password = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    println!("is_valide_password: {}", is_valide_password);

    Ok(is_valide_password)
}
