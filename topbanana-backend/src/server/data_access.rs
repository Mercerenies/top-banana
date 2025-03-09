
use crate::db::models;
use super::auth::DeveloperUser;
use super::error::ApiError;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trait for objects which have a developer that owns them.
///
/// Useful implementors:
///
/// * A [`models::Developer`] owns himself.
///
/// * A [`Uuid`] is owned by the developer it refers to.
///
/// * Any type `T` can be tagged with a developer [`Uuid`], so that
/// the tuple `(T, Uuid)` is considered owned.
pub trait DeveloperOwned {
  fn get_developer_uuid(&self) -> &Uuid;

  /// Check permissions of the requesting user. If the requesting user
  /// is an administrator, then they have full permission to access
  /// any object. Otherwise, the user is only given permission if they
  /// are the owner of the object.
  fn check_permission(object: Option<Self>, requesting_user: &DeveloperUser) -> Result<Self, ApiError>
  where Self: Sized {
    if requesting_user.is_admin() {
      return object.ok_or(ApiError::not_found());
    }
    if let Some(object) = object {
      if requesting_user.user_uuid() == object.get_developer_uuid() {
        return Ok(object);
      }
    }
    Err(ApiError::forbidden())
  }
}

/// Extension trait for `Option<T>` where `T` implements [`DeveloperOwned`].
pub trait DeveloperOwnedExt: Sized {
  type Target: DeveloperOwned;

  fn check_permission(self, requesting_user: &DeveloperUser) -> Result<Self::Target, ApiError>;
}

impl DeveloperOwned for models::Developer {
  fn get_developer_uuid(&self) -> &Uuid {
    &self.developer_uuid
  }
}

impl DeveloperOwned for Uuid {
  fn get_developer_uuid(&self) -> &Uuid {
    self
  }
}

impl<T> DeveloperOwned for (T, Uuid) {
  fn get_developer_uuid(&self) -> &Uuid {
    &self.1
  }
}

impl<T: DeveloperOwned + Sized> DeveloperOwnedExt for Option<T> {
  type Target = T;

  fn check_permission(self, requesting_user: &DeveloperUser) -> Result<T, ApiError> {
    T::check_permission(self, requesting_user)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperResponse {
  pub developer_uuid: Uuid,
  pub name: String,
  pub email: String,
  pub url: Option<String>,
  pub is_admin: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameDao {
  pub developer_uuid: Uuid,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameResponse {
  pub developer_uuid: Uuid,
  pub game_uuid: Uuid,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub game_secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewHighscoreTableDao {
  pub game_uuid: Uuid,
  pub name: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub maximum_scores_retained: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighscoreTableResponse {
  pub game_uuid: Uuid,
  pub table_uuid: Uuid,
  pub name: String,
  pub maximum_scores_retained: Option<i32>,
}

impl DeveloperResponse {
  /// Removes the API key from the response.
  pub fn without_api_key(mut self) -> Self {
    self.api_key = None;
    self
  }
}

impl GameResponse {
  /// Removes the secret key from the response.
  pub fn without_secret_key(mut self) -> Self {
    self.game_secret_key = None;
    self
  }
}

impl From<models::Developer> for DeveloperResponse {
  fn from(d: models::Developer) -> Self {
    Self {
      developer_uuid: d.developer_uuid,
      name: d.name,
      email: d.email,
      url: d.url,
      is_admin: d.is_admin,
      api_key: d.api_key,
    }
  }
}

impl From<models::NewDeveloper> for DeveloperResponse {
  fn from(d: models::NewDeveloper) -> Self {
    Self {
      developer_uuid: d.developer_uuid,
      name: d.name,
      email: d.email,
      url: d.url,
      is_admin: d.is_admin,
      api_key: d.api_key,
    }
  }
}
