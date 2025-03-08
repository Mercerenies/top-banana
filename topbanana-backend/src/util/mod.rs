
pub mod header;

use rand::{TryRngCore, CryptoRng};
use rand::rngs::OsRng;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

/// Generates a base64 encoding of a random sequence of bytes,
/// appropriate for use as an API key or a secret key. Uses the
/// operating system's default source of randomness.
pub fn generate_key() -> String {
  generate_key_with(&mut OsRng.unwrap_err())
}

/// Generates a base64 encoding of a random sequence of bytes,
/// appropriate for use as an API key or a secret key.
pub fn generate_key_with(rng: &mut impl CryptoRng) -> String {
  let mut bytes = [0u8; 64];
  rng.fill_bytes(&mut bytes);
  URL_SAFE_NO_PAD.encode(bytes)
}
