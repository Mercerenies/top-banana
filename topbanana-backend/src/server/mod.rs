
pub mod admin;
pub mod api;
pub mod auth;
pub mod data_access;
pub mod db;
pub mod error;
pub mod openapi;
pub mod requests;

use rocket::{Rocket, Build, Ignite};
use rocket_db_pools::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub async fn run_server() -> Result<Rocket<Ignite>, rocket::Error> {
  build_rocket().launch().await
}

pub fn build_rocket() -> Rocket<Build> {
  rocket::build()
    .mount("/api", api::api_routes())
    .mount("/", SwaggerUi::new("/swagger-ui/<_..>").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
    .attach(db::Db::init())
    .register("/api", error::catchers())
}
