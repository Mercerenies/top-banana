
pub mod auth;
pub mod db;
pub mod error;

use error::{ApiError, ApiSuccessResponse};
use auth::{create_jwt_for_api_key, AuthError};

use rocket::{Route, Rocket, Build, Ignite, routes, post};
use rocket_db_pools::{Database, Connection};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
  pub token: String,
}

pub async fn run_server() -> Result<Rocket<Ignite>, rocket::Error> {
  build_rocket().launch().await
}

pub fn build_rocket() -> Rocket<Build> {
  rocket::build()
    .mount("/api", api_routes())
    .attach(db::Db::init())
    .register("/api", error::catchers())
}

pub fn api_routes() -> Vec<Route> {
  routes![
    authorize,
  ]
}

#[post("/authorize")]
async fn authorize(api_key: auth::XApiKey<'_>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<AuthResponse>, ApiError> {
  let jwt_token = create_jwt_for_api_key(&api_key.0, &mut db).await.map_err(|err| {
    match err {
      AuthError::InvalidApiKey => ApiError::bad_request("Invalid API key"),
      err => ApiError::internal_server_error(&err.to_string()),
    }
  })?;
  Ok(ApiSuccessResponse::new(AuthResponse { token: jwt_token }))
}
