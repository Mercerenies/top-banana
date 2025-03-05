
pub mod auth;
pub mod db;
pub mod error;

use rocket::{Route, Rocket, Build, Ignite, routes, post};

pub async fn run_server() -> Result<Rocket<Ignite>, rocket::Error> {
  build_rocket().launch().await
}

pub fn build_rocket() -> Rocket<Build> {
  rocket::build()
    .mount("/api", api_routes())
}

pub fn api_routes() -> Vec<Route> {
  routes![
    authorize,
  ]
}

#[post("/authorize")]
async fn authorize(api_key: auth::XApiKey<'_>) -> String {
  String::from("Test")
}
