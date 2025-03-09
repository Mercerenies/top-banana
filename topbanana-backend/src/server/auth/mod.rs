
//! Authorization primitives for TopBanana Rocket API.

mod header;
mod jwt;

pub use header::{XApiKey, X_API_KEY_HEADER};
pub use jwt::{create_token, verify_token, JwtClaim, JwtError, UserFlags};

use crate::db::schema::developers;
use crate::util::header::Authorization;
use super::error::ApiError;

use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use thiserror::Error;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use std::str::FromStr;
use std::convert::AsRef;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AuthError {
  #[error("{0}")]
  JwtError(#[from] JwtError),
  #[error("{0}")]
  DieselError(#[from] diesel::result::Error),
  #[error("Invalid API key")]
  InvalidApiKey,
}

/// Rocket request guard that requires an `Authorization: Bearer xxx`
/// header containing a valid JWT token.
#[derive(Debug, Clone)]
pub struct DeveloperUser {
  claim: JwtClaim,
}

/// Rocket request guard that requires an `Authorization: Bearer xxx`
/// header specifically for an admin user.
///
/// The requests accepted by this guard are strictly a subset of those
/// accepted by [`DeveloperUser`].
#[derive(Debug, Clone)]
pub struct AdminUser {
  claim: JwtClaim,
}

/// Subset of the `Developer` model containing the columns needed to
/// generate a JWT token.
#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::developers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DeveloperPerms {
  pub developer_uuid: Uuid,
  pub is_admin: bool,
}

pub const MISSING_AUTH_HEADER: &str = "Missing Authorization header";
pub const INVALID_AUTH_HEADER: &str = "Invalid Authorization header";

pub async fn create_jwt_for_api_key(api_key: &str, db: &mut AsyncPgConnection) -> Result<String, AuthError> {
  let perms = developers::table.filter(developers::api_key.eq(api_key))
    .select(DeveloperPerms::as_select())
    .first(db)
    .await
    .optional()?;
  let Some(perms) = perms else {
    return Err(AuthError::InvalidApiKey);
  };
  let user_flags = perms.user_flags();
  let token = create_token(&perms.developer_uuid, user_flags)?;
  Ok(token)
}

impl DeveloperUser {
  pub fn user_uuid(&self) -> &Uuid {
    &self.claim.sub
  }
}

impl AdminUser {
  pub fn user_uuid(&self) -> &Uuid {
    &self.claim.sub
  }
}

impl DeveloperPerms {
  fn user_flags(&self) -> UserFlags {
    if self.is_admin {
      UserFlags::ADMIN
    } else {
      UserFlags::empty()
    }
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DeveloperUser {
  type Error = ApiError;

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ApiError> {
    let Some(auth_header) = req.headers().get_one("Authorization")
      .and_then(|value| Authorization::from_str(value).ok()) else {
        return request::Outcome::Error((Status::Unauthorized, ApiError::unauthorized(MISSING_AUTH_HEADER)));
      };
    if auth_header.scheme != "Bearer" {
      return request::Outcome::Error((Status::Unauthorized, ApiError::unauthorized(INVALID_AUTH_HEADER)));
    }
    let token = auth_header.params;
    let Ok(claim) = verify_token(&token) else {
      return request::Outcome::Error((Status::Unauthorized, ApiError::unauthorized(INVALID_AUTH_HEADER)));
    };
    request::Outcome::Success(DeveloperUser { claim })
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
  type Error = ApiError;

  async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, ApiError> {
    let developer = match DeveloperUser::from_request(req).await {
      request::Outcome::Success(developer) => developer,
      request::Outcome::Error(f) => return request::Outcome::Error(f),
      request::Outcome::Forward(f) => return request::Outcome::Forward(f),
    };
    if !developer.claim.user_flags.contains(UserFlags::ADMIN) {
      return request::Outcome::Error((Status::Forbidden, ApiError::forbidden("Forbidden")));
    }
    request::Outcome::Success(AdminUser { claim: developer.claim })
  }
}

impl AsRef<JwtClaim> for DeveloperUser {
  fn as_ref(&self) -> &JwtClaim {
    &self.claim
  }
}

impl AsRef<JwtClaim> for AdminUser {
  fn as_ref(&self) -> &JwtClaim {
    &self.claim
  }
}
