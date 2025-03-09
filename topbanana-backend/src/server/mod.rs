
pub mod admin;
pub mod auth;
pub mod data_access;
pub mod db;
pub mod error;

use error::{ApiError, ApiSuccessResponse};
use auth::{create_jwt_for_api_key, DeveloperUser, AuthError};
use data_access::{DeveloperOwnedExt, DeveloperResponse, NewGameDao, GameResponse, NewHighscoreTableDao, HighscoreTableResponse};
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
    create_highscore_table,
    get_highscore_table,
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
    .optional()?
    .check_permission(&requesting_user)?;
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
  let (game, developer_uuid) = schema::games::table
    .filter(schema::games::game_uuid.eq(&*uuid))
    .inner_join(schema::developers::table)
    .select((schema::games::all_columns, schema::developers::developer_uuid))
    .first::<(models::Game, Uuid)>(&mut db)
    .await
    .optional()?
    .check_permission(&requesting_user)?;

  let game_response = GameResponse {
    developer_uuid,
    game_uuid: game.game_uuid,
    name: game.name,
    game_secret_key: None,
  };
  Ok(ApiSuccessResponse::new(game_response))
}

#[post("/highscore-table", data = "<params>")]
async fn create_highscore_table(requesting_user: DeveloperUser, params: Json<NewHighscoreTableDao>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<HighscoreTableResponse>, ApiError> {
  let params = params.0;
  let (game_id, _) = schema::games::table
    .filter(schema::games::game_uuid.eq(&params.game_uuid))
    .inner_join(schema::developers::table)
    .select((schema::games::id, schema::developers::developer_uuid))
    .first::<(i32, Uuid)>(&mut db)
    .await
    .optional()?
    .check_permission(&requesting_user)?;

  let new_highscore_table = models::NewHighscoreTable {
    game_id,
    name: params.name,
    table_uuid: Uuid::new_v4(),
    maximum_scores_retained: params.maximum_scores_retained,
  };
  diesel::insert_into(schema::highscore_tables::table)
    .values(&new_highscore_table)
    .execute(&mut db)
    .await
    .map_err(ApiError::from_on_create)?;

  let response = HighscoreTableResponse {
    game_uuid: params.game_uuid,
    table_uuid: new_highscore_table.table_uuid,
    name: new_highscore_table.name,
    maximum_scores_retained: new_highscore_table.maximum_scores_retained,
  };
  Ok(ApiSuccessResponse::new(response))
}

#[get("/highscore-table/<uuid>")]
async fn get_highscore_table(requesting_user: DeveloperUser, uuid: ParamFromStr<Uuid>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<HighscoreTableResponse>, ApiError> {
  let ((highscore_table, game_uuid), _developer_uuid) = schema::highscore_tables::table
    .filter(schema::highscore_tables::table_uuid.eq(&*uuid))
    .inner_join(schema::games::table.inner_join(schema::developers::table))
    .select(((schema::highscore_tables::all_columns, schema::games::game_uuid), schema::developers::developer_uuid))
    .first::<((models::HighscoreTable, Uuid), Uuid)>(&mut db)
    .await
    .optional()?
    .check_permission(&requesting_user)?;
  let response = HighscoreTableResponse {
    game_uuid,
    table_uuid: highscore_table.table_uuid,
    name: highscore_table.name,
    maximum_scores_retained: highscore_table.maximum_scores_retained,
  };
  Ok(ApiSuccessResponse::new(response))
}
