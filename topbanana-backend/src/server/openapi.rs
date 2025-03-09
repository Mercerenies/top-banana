
use utoipa::{Modify, openapi};
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue};

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut openapi::OpenApi) {
    let mut components = openapi.components.take().unwrap_or_default();

    let api_key = SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key")));
    components.add_security_scheme("X-Api-Key", api_key);
    openapi.components = Some(components);
  }
}
