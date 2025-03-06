
//! Authorization primitives for TopBanana Rocket API.

mod header;
mod jwt;

pub use header::{XApiKey, X_API_KEY_HEADER};
pub use jwt::{create_token, verify_token, JwtClaim, JwtError, UserFlags};

use crate::db::schema::developers;

use thiserror::Error;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

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

/// Subset of the `Developer` model containing the columns needed to
/// generate a JWT token.
#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::developers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct DeveloperPerms {
  pub developer_uuid: Uuid,
  pub is_admin: bool,
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
