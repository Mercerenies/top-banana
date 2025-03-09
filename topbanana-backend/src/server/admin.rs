
use crate::db::schema;
use crate::db::models::NewDeveloper;
use crate::util::generate_key;
use super::data_access::DeveloperResponse;
use super::db::Db;
use super::auth::AdminUser;
use super::error::{ApiSuccessResponse, ApiError};

use rocket::post;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use diesel_async::RunQueryDsl;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDeveloperParams {
  pub name: String,
  pub email: String,
  #[serde(default)]
  pub url: Option<String>,
}

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
