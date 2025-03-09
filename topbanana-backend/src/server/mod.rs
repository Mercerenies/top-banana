
pub mod admin;
pub mod auth;
pub mod data_access;
pub mod db;
pub mod error;

use error::{ApiError, ApiSuccessResponse};
use auth::{create_jwt_for_api_key, DeveloperUser, AuthError};
use data_access::{DeveloperResponse, NewGameDao, GameResponse};
use crate::db::{schema, models};
use crate::util::{ParamFromStr, generate_key};

use rocket::{Route, Rocket, Build, Ignite, routes, post, get};
use rocket::serde::json::Json;
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
    create_game,
    get_game,
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

#[post("/game", data = "<params>")]
async fn create_game(requesting_user: DeveloperUser, params: Json<NewGameDao>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<GameResponse>, ApiError> {
  let params = params.0;
  if !requesting_user.is_admin() && &params.developer_uuid != requesting_user.user_uuid() {
    return Err(ApiError::forbidden());
  }
  let developer_id = schema::developers::table
    .filter(schema::developers::developer_uuid.eq(&params.developer_uuid))
    .select(schema::developers::id)
    .first::<i32>(&mut db)
    .await
    .map_err(ApiError::from_on_create)?;

  let new_game = models::NewGame {
    developer_id,
    game_uuid: Uuid::new_v4(),
    game_secret_key: generate_key(),
    name: params.name,
  };
  diesel::insert_into(schema::games::table)
    .values(&new_game)
    .execute(&mut db)
    .await
    .map_err(ApiError::from_on_create)?;

  let game_response = GameResponse {
    developer_uuid: params.developer_uuid,
    game_uuid: new_game.game_uuid,
    name: new_game.name,
    game_secret_key: Some(new_game.game_secret_key),
  };
  Ok(ApiSuccessResponse::new(game_response))
}

#[get("/game/<uuid>")]
async fn get_game(requesting_user: DeveloperUser, uuid: ParamFromStr<Uuid>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<GameResponse>, ApiError> {
  let game = schema::games::table
    .filter(schema::games::game_uuid.eq(&*uuid))
    .inner_join(schema::developers::table)
    .select((schema::games::all_columns, schema::developers::developer_uuid))
    .first::<(models::Game, Uuid)>(&mut db)
    .await
    .optional()?;
  let (game, developer_uuid) = check_game_perms(&requesting_user, game)?;

  let game_response = GameResponse {
    developer_uuid,
    game_uuid: game.game_uuid,
    name: game.name,
    game_secret_key: None,
  };
  Ok(ApiSuccessResponse::new(game_response))
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

/// Returns the matching game, if they exist and the requesting user
/// has permission to see them.
fn check_game_perms(requesting_user: &DeveloperUser, matching_game: Option<(models::Game, Uuid)>) -> Result<(models::Game, Uuid), ApiError> {
  if requesting_user.is_admin() {
    // Admin has full permission to access everything.
    return matching_game.ok_or(ApiError::not_found());
  }

  // Otherwise, a user only has permission to access their own games.
  if let Some((matching_game, developer_uuid)) = matching_game {
    if requesting_user.user_uuid() == &developer_uuid {
      return Ok((matching_game, developer_uuid));
    }
  }
  Err(ApiError::forbidden())
}
