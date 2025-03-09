
use utoipa::{Modify, ToSchema, openapi};
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityRequirement};
use uuid::Uuid;

pub struct SecurityAddon;

/// [`Uuid`] does not implement [`ToSchema`], so we use this type as
/// documentation for any OpenAPI responses or parameters that contain
/// a value of type `Uuid`. Note that this type is ONLY used for
/// OpenAPI documentation, not at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[schema(value_type = String, examples("f1aa6898-6294-44c6-a9a4-cd599e7849b8"))]
pub struct OpenApiUuid(pub Uuid);

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut openapi::OpenApi) {
    let mut components = openapi.components.take().unwrap_or_default();

    let api_key = SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key")));
    components.add_security_scheme("X-Api-Key", api_key);

    let jwt = SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer));
    components.add_security_scheme("Bearer", jwt);

    openapi.components = Some(components);
    openapi.security = Some(vec![SecurityRequirement::new("Bearer".to_string(), Vec::<String>::new())]);
  }
}
