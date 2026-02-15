
use crate::db::schema;
use crate::db::models::NewDeveloper;
use crate::util::generate_key;
use super::data_access::DeveloperResponse;
use super::db::Db;
use super::auth::AdminUser;
use super::error::{ApiSuccessResponse, ApiSuccessResponseBody, ApiError};

use rocket::post;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use diesel_async::RunQueryDsl;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewDeveloperParams {
  /// The new developer's user-friendly name.
  pub name: String,
  /// New developer's email address.
  pub email: String,
  /// A URL for the developer's website, optional.
  #[serde(default)]
  pub url: Option<String>,
}

/// Creates a new developer user.
///
/// This endpoint is only available to administrators. The returned
/// API key cannot be accessed from the API after creation.
#[utoipa::path(
  post,
  path="/api/developer",
  tag="developer",
  responses(
    (status = 200, description = "Developer created successfully", body = ApiSuccessResponseBody<DeveloperResponse>),
    (status = 409, description = "Developer with provided arguments already exists"),
  )
)]
#[post("/developer", data = "<params>")]
pub async fn create_developer(
  _admin_user: AdminUser,
  params: Json<NewDeveloperParams>,
  mut db: Connection<Db>,
) -> Result<ApiSuccessResponse<DeveloperResponse>, ApiError> {
  let Json(params) = params;
  let developer_uuid = Uuid::new_v4();
  let api_key = generate_key();
  let new_developer = NewDeveloper {
    developer_uuid,
    name: params.name,
    email: params.email,
    url: params.url,
    is_admin: false,
    api_key: Some(api_key),
  };
  diesel::insert_into(schema::developers::table)
    .values(&new_developer)
    .execute(&mut db)
    .await
    .map_err(ApiError::from_on_create)?;
  Ok(ApiSuccessResponse::new(new_developer.into()))
}
