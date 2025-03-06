
use crate::server::error::ApiError;

use rocket::request::{self, Request, FromRequest};

/// Rocket request guard type to query the X-Api-Key header.
#[derive(Debug, Clone)]
pub struct XApiKey<'r>(pub &'r str);

pub const X_API_KEY_HEADER: &str = "X-Api-Key";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for XApiKey<'r> {
  type Error = ApiError;

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ApiError> {
    match req.headers()
      .get_one(X_API_KEY_HEADER)
      .map(XApiKey)
      .ok_or_else(|| ApiError::bad_request("Missing X-Api-Key header")) {
        Err(err) => request::Outcome::Error((err.status(), err)),
        Ok(ok) => request::Outcome::Success(ok),
      }
  }
}
