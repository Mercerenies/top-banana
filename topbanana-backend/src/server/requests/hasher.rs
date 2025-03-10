
use digest::Digest;
use sha1::Sha1;
use sha2::Sha256;

/// A type capable of signing request payloads.
pub trait RequestSigningHasher {
  fn security_level(&self) -> SecurityLevel;

  fn apply_hash(&self, buf: &str) -> Box<[u8]>;
}

#[derive(Debug, Clone)]
pub struct Sha256Hasher;

#[derive(Debug, Clone)]
pub struct Sha1Hasher;

/// Security level of various hashing algorithms.
///
/// Some game engines only support older hashing algorithms, so we
/// make the security level configurable so that developers wishing to
/// support such engines can voluntarily support older hashing
/// functions, while those who don't need the legacy support can
/// maintain a higher security model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
  /// Low-security hash functions, including functions that have been
  /// effectively broken.
  Low,
  /// High-security fast hash functions.
  High,
}

impl RequestSigningHasher for Sha256Hasher {
  fn security_level(&self) -> SecurityLevel {
    SecurityLevel::High
  }

  fn apply_hash(&self, buf: &str) -> Box<[u8]> {
    let mut hasher = Sha256::new();
    hasher.update(buf.as_bytes());
    hasher.finalize().into_iter().collect()
  }
}

impl RequestSigningHasher for Sha1Hasher {
  fn security_level(&self) -> SecurityLevel {
    SecurityLevel::Low
  }

  fn apply_hash(&self, buf: &str) -> Box<[u8]> {
    let mut hasher = Sha1::new();
    hasher.update(buf.as_bytes());
    hasher.finalize().into_iter().collect()
  }
}
