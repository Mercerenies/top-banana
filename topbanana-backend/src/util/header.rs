
//! Utility functions for HTTP headers.

use thiserror::Error;

use std::fmt::{self, Display};
use std::str::FromStr;

/// Rust-side representation of the HTTP "Authorization" header.
#[derive(Debug, Clone)]
pub struct Authorization {
  pub scheme: String,
  pub params: String,
}

#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum AuthorizationParseError {
  #[error("Could not find authorization scheme")]
  MissingScheme,
}

impl Display for Authorization {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}", self.scheme, self.params)
  }
}

impl FromStr for Authorization {
  type Err = AuthorizationParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Some((scheme, params)) = s.split_once(' ') {
      Ok(Authorization {
        scheme: scheme.to_owned(),
        params: params.to_owned(),
      })
    } else {
      Err(AuthorizationParseError::MissingScheme)
    }
  }
}
