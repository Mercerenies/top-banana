
use crate::db::schema;
use crate::db::models::NewDeveloper;
use crate::util::generate_key;
use super::db::Db;
use super::auth::AdminUser;
use super::error::{ApiSuccessResponse, ApiError};

use rocket::{Route, routes, post};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDeveloperResponse {
  pub developer_uuid: Uuid,
  pub api_key: String,
}

pub fn admin_routes() -> Vec<Route> {
  routes![create_developer]
}

#[post("/developer", data = "<params>")]
async fn create_developer(
  _admin_user: AdminUser,
  params: Json<NewDeveloperParams>,
  mut db: Connection<Db>,
) -> Result<ApiSuccessResponse<NewDeveloperResponse>, ApiError> {
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
  let resp = NewDeveloperResponse {
    developer_uuid: new_developer.developer_uuid,
    api_key: new_developer.api_key.unwrap(),
  };
  Ok(ApiSuccessResponse::new(resp))
}
