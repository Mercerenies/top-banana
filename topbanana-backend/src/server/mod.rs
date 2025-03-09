
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

#[derive(Debug, Clone, Serialize)]
pub struct ScoresResponse {
  pub scores: Vec<ScoresResponseEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoresResponseEntry {
  pub player_name: String,
  pub player_score: f64,
  pub player_score_metadata: Option<String>,
  #[serde(serialize_with = "serialize_datetime")]
  pub creation_timestamp: chrono::NaiveDateTime,
}

impl From<models::HighscoreTableEntry> for ScoresResponseEntry {
  fn from(entry: models::HighscoreTableEntry) -> Self {
    Self {
      player_name: entry.player_name,
      player_score: entry.player_score,
      player_score_metadata: entry.player_score_metadata,
      creation_timestamp: entry.creation_timestamp,
    }
  }
}

fn serialize_datetime<S>(datetime: &chrono::NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where S: serde::Serializer {
  let formatted = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
  serializer.serialize_str(&formatted)
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
    get_highscore_table_scores,
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

#[get("/highscore-table/<uuid>/scores")]
async fn get_highscore_table_scores(
  requesting_user: DeveloperUser,
  uuid: ParamFromStr<Uuid>,
  mut db: Connection<db::Db>,
) -> Result<ApiSuccessResponse<ScoresResponse>, ApiError> {
  let (highscore_table_id, _developer_uuid) = schema::highscore_tables::table
    .filter(schema::highscore_tables::table_uuid.eq(&*uuid))
    .inner_join(schema::games::table.inner_join(schema::developers::table))
    .select((schema::highscore_tables::id, schema::developers::developer_uuid))
    .first::<(i32, Uuid)>(&mut db)
    .await
    .optional()?
    .check_permission(&requesting_user)?;
  let entries = schema::highscore_table_entries::table
    .filter(schema::highscore_table_entries::highscore_table_id.eq(highscore_table_id))
    .order(schema::highscore_table_entries::player_score.desc())
    .load::<models::HighscoreTableEntry>(&mut db)
    .await?;
  let entries = entries.into_iter().map(ScoresResponseEntry::from).collect();
  Ok(ApiSuccessResponse::new(ScoresResponse { scores: entries }))
}
