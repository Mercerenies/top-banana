
pub mod header;

use rand::{TryRngCore, CryptoRng};
use rand::rngs::OsRng;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rocket::Request;
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::data::{self, Data, FromData};

use std::str::FromStr;
use std::fmt::Debug;
use std::ops::Deref;
use std::error::{Error as StdError};
use std::io;

/// Newtype wrapper which converts a [`FromStr`] impl into a
/// [`FromParam`] impl.
#[derive(Debug, Clone)]
pub struct ParamFromStr<T>(pub T);

/// Newtype wrapper which converts a [`FromStr`] impl into a
/// [`FromData`] impl.
///
/// The `FromData` impl is subject to the Rocket string datatype
/// limit, for security reasons.
#[derive(Debug, Clone)]
pub struct DataFromStr<T>(pub T);

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

impl<'a, T> FromParam<'a> for ParamFromStr<T>
where T: FromStr,
      <T as FromStr>::Err: Debug {
  type Error = <T as FromStr>::Err;

  fn from_param(param: &'a str) -> Result<Self, <T as FromStr>::Err> {
    Ok(ParamFromStr(param.parse()?))
  }
}

#[rocket::async_trait]
impl<'r, T> FromData<'r> for DataFromStr<T>
where T: FromStr,
      <T as FromStr>::Err: StdError + Send + Sync + 'static {
  type Error = io::Error;

  async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
    <&str>::from_data(req, data).await.and_then(|data| {
      match data.parse() {
        Ok(param) => data::Outcome::Success(DataFromStr(param)),
        Err(err) => data::Outcome::Error((Status::BadRequest, io::Error::other(err))),
      }
    })
  }
}

impl<T> Deref for ParamFromStr<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> Deref for DataFromStr<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
