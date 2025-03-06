
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use bitflags::bitflags;
use thiserror::Error;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Validation, Header};

use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct JwtClaim {
  /// The user's UUID being claimed.
  pub sub: Uuid,
  /// Flags associated with the user.
  pub user_flags: UserFlags,
  /// Expiration time, in seconds since the Unix epoch.
  pub exp: usize,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum JwtError {
  #[error("{0}")]
  JsonWebTokenError(#[from] jsonwebtoken::errors::Error),
  #[error("Missing JWT_SECRET_KEY environment variable")]
  MissingJwtSecretKeyEnvVar,
}

pub const SECRET_KEY_ENV_VAR: &str = "JWT_SECRET_KEY";
pub const JWT_EXPIRATION_TIME: chrono::Duration = chrono::Duration::hours(1);

bitflags! {
  #[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize)]
  pub struct UserFlags: u32 {
    const ADMIN = 0b00000001;
  }
}

pub fn create_token(user_uuid: &Uuid, user_flags: UserFlags) -> Result<String, JwtError> {
  let claim = JwtClaim {
    sub: user_uuid.to_owned(),
    user_flags,
    exp: (chrono::Utc::now() + JWT_EXPIRATION_TIME).timestamp() as usize,
  };
  let encoding_key = EncodingKey::from_base64_secret(&get_secret_key()?)?;
  let token = encode(
    &Header::default(),
    &claim,
    &encoding_key,
  )?;
  Ok(token)
}

pub fn verify_token(token_str: &str) -> Result<Uuid, JwtError> {
  let decoding_key = DecodingKey::from_base64_secret(&get_secret_key()?)?;
  let claims = decode::<JwtClaim>(
    token_str,
    &decoding_key,
    &Validation::default(),
  )?;
  Ok(claims.claims.sub)
}

fn get_secret_key() -> Result<String, JwtError> {
  env::var(SECRET_KEY_ENV_VAR)
    .map_err(|_| JwtError::MissingJwtSecretKeyEnvVar)
}
