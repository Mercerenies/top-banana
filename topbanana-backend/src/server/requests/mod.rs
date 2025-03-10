
//! Helpers for verifying request UUID and digital signature
//! information.

mod hasher;

pub use hasher::{RequestSigningHasher, SecurityLevel, Sha256Hasher, Sha1Hasher};

use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use thiserror::Error;
use serde::de::DeserializeOwned;

use std::str::{from_utf8, Utf8Error, FromStr};

/// A payload for a request made from a relevant video game client.
///
/// Payloads of this form consist of two base64url-encoded strings,
/// separated by a dot. The first string is the actual payload, and
/// the second is the digital signature.
///
/// Existence of this structure does NOT guarantee that the signature
/// has been verified. It is possible for this structure to contain
/// unverified (and potentially invalid) signatures.
#[derive(Debug, Clone)]
pub struct GameRequestPayload {
  payload_base64: String,
  signature_base64: String,
}

#[derive(Debug, Clone, Error)]
#[error("Invalid GameRequestPayload")]
pub struct GameRequestPayloadFromStrError {
  _priv: (),
}

#[derive(Debug, Clone, Error)]
#[error("Invalid request signature")]
pub struct VerificationError {
  _priv: (),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeserializeError {
  #[error("{0}")]
  JsonError(#[from] serde_json::Error),
  #[error("{0}")]
  Base64Error(#[from] base64::DecodeError),
  #[error("{0}")]
  Utf8Error(#[from] Utf8Error),
}

impl GameRequestPayload {
  pub fn new(payload_base64: String, signature_base64: String) -> Self {
    Self {
      payload_base64,
      signature_base64,
    }
  }

  pub fn verify(&self, secret_key: &str, hasher: &impl RequestSigningHasher) -> Result<(), VerificationError> {
    let full_payload = format!("{}.{}", self.payload_base64, secret_key);
    let expected_signature = hasher.apply_hash(&full_payload);
    let given_signature = URL_SAFE.decode(self.signature_base64.as_bytes()).map_err(|_| VerificationError { _priv: () })?;
    if expected_signature.as_ref() != given_signature.as_slice() {
      return Err(VerificationError { _priv: () });
    }
    Ok(())
  }

  pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, DeserializeError> {
    let payload = URL_SAFE.decode(&self.payload_base64)?;
    let payload = serde_json::from_str(from_utf8(&payload)?)?;
    Ok(payload)
  }
}

impl FromStr for GameRequestPayload {
  type Err = GameRequestPayloadFromStrError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let Some((payload_base64, signature_base64)) = s.split_once('.') else {
      return Err(GameRequestPayloadFromStrError { _priv: () });
    };
    Ok(GameRequestPayload::new(payload_base64.to_string(), signature_base64.to_string()))
  }
}
