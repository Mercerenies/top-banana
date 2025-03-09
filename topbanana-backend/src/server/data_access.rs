
use crate::db::models;
use super::auth::DeveloperUser;
use super::error::ApiError;
use super::openapi::OpenApiUuid;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeveloperResponse {
  #[schema(value_type = OpenApiUuid)]
  pub developer_uuid: Uuid,
  pub name: String,
  pub email: String,
  pub url: Option<String>,
  #[schema(examples("false"))]
  pub is_admin: bool,
  /// The API key is only supplied upon initial user creation and
  /// cannot be recovered after the fact.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewGameDao {
  /// Non-admin users can only create games belonging to themselves.
  /// If you are not an admin, then `developer_uuid` must be your own
  /// UUID.
  #[schema(value_type = OpenApiUuid)]
  pub developer_uuid: Uuid,
  pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameResponse {
  /// The developer who owns this game.
  #[schema(value_type = OpenApiUuid)]
  pub developer_uuid: Uuid,
  #[schema(value_type = OpenApiUuid)]
  pub game_uuid: Uuid,
  pub name: String,
  /// The game's secret key is only supplied upon initial game
  /// creation and cannot be recovered after the fact.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub game_secret_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewHighscoreTableDao {
  /// The game that this table belongs to.
  #[schema(value_type = OpenApiUuid)]
  pub game_uuid: Uuid,
  pub name: String,
  /// Maximum number of scores retained by this highscore table. Omit
  /// to keep all scores. Administrators may choose to limit the
  /// maximum value of this field.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub maximum_scores_retained: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HighscoreTableResponse {
  /// The game that this table belongs to.
  #[schema(value_type = OpenApiUuid)]
  pub game_uuid: Uuid,
  #[schema(value_type = OpenApiUuid)]
  pub table_uuid: Uuid,
  pub name: String,
  /// The maximum number of scores retained by this highscore table.
  /// If this field is `null`, then there is no limit.
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
