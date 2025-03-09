
use super::{admin, api};

use utoipa::{Modify, OpenApi, ToSchema, openapi};
use utoipa::openapi::security::{SecurityScheme, ApiKey, ApiKeyValue, Http, HttpAuthScheme, SecurityRequirement};
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
  paths(
    api::authorize,
    admin::create_developer, api::get_developer, api::get_current_developer,
    api::create_game, api::get_game,
    api::create_highscore_table, api::get_highscore_table, api::get_highscore_table_scores,
  ),
  tags(
    (name = "authorization", description = "Authorization API for developers"),
    (name = "developer", description = "Query information about individual developers"),
    (name = "game", description = "Video game access and creation"),
    (name = "highscore-table", description = "Highscore table access and creation"),
  ),
  modifiers(&SecurityAddon),
  components(),
)]
pub struct ApiDoc;

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
