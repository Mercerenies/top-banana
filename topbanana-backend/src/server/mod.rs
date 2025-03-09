
pub mod admin;
pub mod auth;
pub mod data_access;
pub mod db;
pub mod error;

use error::{ApiError, ApiSuccessResponse};
use auth::{create_jwt_for_api_key, DeveloperUser, AuthError};
use data_access::DeveloperResponse;
use crate::db::{schema, models};
use crate::util::ParamFromStr;

use rocket::{Route, Rocket, Build, Ignite, routes, post, get};
use rocket_db_pools::{Database, Connection};
use serde::Serialize;
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
  pub token: String,
}

pub async fn run_server() -> Result<Rocket<Ignite>, rocket::Error> {
  build_rocket().launch().await
}

pub fn build_rocket() -> Rocket<Build> {
  let mut base_api_routes = Vec::new();
  base_api_routes.extend(api_routes());
  base_api_routes.extend(admin::admin_routes());

  rocket::build()
    .mount("/api", base_api_routes)
    .attach(db::Db::init())
    .register("/api", error::catchers())
}

pub fn api_routes() -> Vec<Route> {
  routes![
    authorize,
    get_developer,
    get_current_developer,
  ]
}

#[post("/authorize")]
async fn authorize(api_key: auth::XApiKey<'_>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<AuthResponse>, ApiError> {
  let jwt_token = create_jwt_for_api_key(&api_key.0, &mut db).await.map_err(|err| {
    match err {
      AuthError::InvalidApiKey => ApiError::bad_request().with_message("Invalid API key"),
      err => ApiError::internal_server_error(&err.to_string()),
    }
  })?;
  Ok(ApiSuccessResponse::new(AuthResponse { token: jwt_token }))
}

#[get("/developer/<uuid>")]
async fn get_developer(requesting_user: DeveloperUser, uuid: ParamFromStr<Uuid>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<DeveloperResponse>, ApiError> {
  let matching_user = schema::developers::table
    .filter(schema::developers::developer_uuid.eq(&*uuid))
    .get_result::<models::Developer>(&mut db)
    .await
    .optional()?;
  let matching_user = check_developer_perms(&requesting_user, matching_user)?;
  Ok(ApiSuccessResponse::new(DeveloperResponse::from(matching_user).without_api_key()))
}

#[get("/developer/me")]
async fn get_current_developer(requesting_user: DeveloperUser, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<DeveloperResponse>, ApiError> {
  let matching_user = schema::developers::table
    .filter(schema::developers::developer_uuid.eq(requesting_user.user_uuid()))
    .get_result::<models::Developer>(&mut db)
    .await?;
  Ok(ApiSuccessResponse::new(DeveloperResponse::from(matching_user).without_api_key()))
}

/// Returns the matching developer, if they exist and the requesting
/// user has permission to see them.
fn check_developer_perms(requesting_user: &DeveloperUser, matching_user: Option<models::Developer>) -> Result<models::Developer, ApiError> {
  if requesting_user.is_admin() {
    // Admin has full permission to access everything.
    return matching_user.ok_or(ApiError::not_found());
  }

  if let Some(matching_user) = matching_user {
    // Non-admin developer can only access themself.
    if requesting_user.user_uuid() == &matching_user.developer_uuid {
      return Ok(matching_user);
    }
  }
  Err(ApiError::forbidden())
}
