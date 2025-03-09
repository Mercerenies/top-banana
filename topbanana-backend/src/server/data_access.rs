
use crate::db::models;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperResponse {
  pub developer_uuid: Uuid,
  pub name: String,
  pub email: String,
  pub url: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub api_key: Option<String>,
}

impl DeveloperResponse {
  /// Removes the API key from the response.
  pub fn without_api_key(mut self) -> Self {
    self.api_key = None;
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
      api_key: d.api_key,
    }
  }
}
