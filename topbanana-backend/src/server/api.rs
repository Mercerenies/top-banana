
//! Endpoints related to the developer API.
//!
//! Note that admin-only endpoints are available at
//! [`admin`](crate::server::admin).

use super::error::{ApiError, ApiSuccessResponse, ApiSuccessResponseBody};
use super::auth::{create_jwt_for_api_key, DeveloperUser, AuthError, XApiKey};
use super::data_access::{DeveloperOwnedExt, DeveloperResponse, NewGameDao, GameResponse, NewHighscoreTableDao, HighscoreTableResponse};
use super::openapi::OpenApiUuid;
use super::{admin, db};
use crate::db::{schema, models};
use crate::util::{ParamFromStr, generate_key};

use rocket::{Route, routes, post, get};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncPgConnection};
use utoipa::ToSchema;
use serde::Serialize;

pub const MAX_HIGHSCORES_RETAINED_FOR_NON_ADMIN: i32 = 100;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
  /// A fresh JWT token associated to the user.
  pub token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScoresResponse {
  /// All highscores in the table, sorted in descending order by score
  /// value. Tied scores are sorted by creation timestamp, with
  /// earlier scores ranking higher.
  pub scores: Vec<ScoresResponseEntry>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScoresResponseEntry {
  /// The name of the player who submitted the score.
  pub player_name: String,
  /// The player's score, as a float.
  pub player_score: f64,
  /// Optional metadata supplied with the player's submission. The
  /// meaning of this field is game-specific.
  pub player_score_metadata: Option<String>,
  /// When the score was submitted.
  #[schema(value_type = String, example = "2025-02-01 05:33:10")]
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

pub fn api_routes() -> Vec<Route> {
  routes![
    authorize,
    admin::create_developer,
    get_developer,
    get_current_developer,
    create_game,
    get_game,
    create_highscore_table,
    get_highscore_table,
    get_highscore_table_scores,
  ]
}

/// Authorizes a developer to perform API calls.
///
/// Takes an API key in the X-Api-Key header and returns a JWT token
/// if successful. The JWT token is valid for one hour after creation
/// and can be used for any of the user-facing API endpoints.
///
/// NOTE: A JWT token is **not** used for game-facing endpoints, only
/// for the user-facing API.
#[utoipa::path(
  post,
  path="/api/authorize",
  tag="authorization",
  security(("X-Api-Key" = [])),
  responses(
    (status = 200, description = "A JWT token", body = ApiSuccessResponseBody<AuthResponse>),
    (status = 400, description = "Invalid API key")
  ),
)]
#[post("/authorize")]
async fn authorize(api_key: XApiKey<'_>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<AuthResponse>, ApiError> {
  let jwt_token = create_jwt_for_api_key(api_key.0, &mut db).await.map_err(|err| {
    match err {
      AuthError::InvalidApiKey => ApiError::bad_request().with_message("Invalid API key"),
      err => ApiError::internal_server_error(err.to_string()),
    }
  })?;
  Ok(ApiSuccessResponse::new(AuthResponse { token: jwt_token }))
}

/// Gets information about the specified user.
///
/// Non-admin users can only query their own information.
#[utoipa::path(
  get,
  path="/api/developer/{uuid}",
  tag="developer",
  params(
    ("uuid" = OpenApiUuid, Path, description = "Developer UUID"),
  ),
  responses(
    (status = 200, description = "Developer information", body = ApiSuccessResponseBody<DeveloperResponse>),
    (status = 403, description = "Forbidden"),
    (status = 404, description = "Developer not found"),
  )
)]
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

/// Gets information about the current user.
#[utoipa::path(
  get,
  path="/api/developer/me",
  tag="developer",
  responses(
    (status = 200, description = "Developer information", body = ApiSuccessResponseBody<DeveloperResponse>),
  )
)]
#[get("/developer/me")]
async fn get_current_developer(requesting_user: DeveloperUser, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<DeveloperResponse>, ApiError> {
  let matching_user = schema::developers::table
    .filter(schema::developers::developer_uuid.eq(requesting_user.user_uuid()))
    .get_result::<models::Developer>(&mut db)
    .await?;
  Ok(ApiSuccessResponse::new(DeveloperResponse::from(matching_user).without_api_key()))
}

/// Creates a new video game.
///
/// The game's returned secret key cannot be accessed after this
/// endpoint returns.
#[utoipa::path(
  post,
  path="/api/game",
  tag="game",
  responses(
    (status = 200, description = "Game created successfully", body = ApiSuccessResponseBody<GameResponse>),
    (status = 403, description = "Not allowed to create a game with these parameters"),
  ),
)]
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
    security_level: params.security_level.unwrap_or_default(),
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
    security_level: new_game.security_level,
  };
  Ok(ApiSuccessResponse::new(game_response))
}

/// Gets details about the video game with the given UUID.
///
/// Admins can query any game, while non-admins can only query their
/// own games.
#[utoipa::path(
  get,
  path="/api/game/{uuid}",
  tag="game",
  params(
    ("uuid" = OpenApiUuid, Path, description = "Game UUID"),
  ),
  responses(
    (status = 200, description = "Game details", body = ApiSuccessResponseBody<GameResponse>),
    (status = 403, description = "Forbidden"),
    (status = 404, description = "Game not found"),
  ),
)]
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
    security_level: game.security_level,
  };
  Ok(ApiSuccessResponse::new(game_response))
}

/// Creates a new highscore table.
///
/// Requesting user must either own the game or be an admin.
#[utoipa::path(
  post,
  path="/api/highscore-table",
  tag="highscore-table",
  responses(
    (status = 200, description = "Highscore table created successfully", body = ApiSuccessResponseBody<HighscoreTableResponse>),
    (status = 403, description = "Forbidden"),
  ),
)]
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
    maximum_scores_retained: normalize_max_scores(params.maximum_scores_retained, &requesting_user),
    unique_entries: params.unique_entries,
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

/// Non-admin users are not permitted to make highscore tables with no
/// limit, or tables with a limit higher than
/// [`MAX_HIGHSCORES_RETAINED_FOR_NON_ADMIN`]. This function enforces
/// that limit. Admin users are not subject to this restriction.
fn normalize_max_scores(maximum_scores_retained: Option<i32>, requesting_user: &DeveloperUser) -> Option<i32> {
  if requesting_user.is_admin() {
    // Implicitly trust admin users. Do not restrict their inputs.
    return maximum_scores_retained;
  }
  let Some(n) = maximum_scores_retained else {
    return Some(MAX_HIGHSCORES_RETAINED_FOR_NON_ADMIN);
  };
  if !(0..=MAX_HIGHSCORES_RETAINED_FOR_NON_ADMIN).contains(&n) {
    return Some(MAX_HIGHSCORES_RETAINED_FOR_NON_ADMIN);
  }
  Some(n)
}

/// Queries the details of a highscore table.
///
/// Requesting user must be an admin or the owner of the game.
#[utoipa::path(
  get,
  path="/api/highscore-table/{uuid}",
  tag="highscore-table",
  params(
    ("uuid" = OpenApiUuid, Path, description = "Highscore table UUID"),
  ),
  responses(
    (status = 200, description = "Highscore table details", body = ApiSuccessResponseBody<HighscoreTableResponse>),
    (status = 403, description = "Forbidden"),
    (status = 404, description = "Highscore table not found"),
  ),
)]
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

/// Returns a list of all highscores on the given table.
///
/// Returned table is sorted from highest to lowest score.
///
/// Requesting user must be an admin or the owner of the game.
#[utoipa::path(
  get,
  path="/api/highscore-table/{uuid}/scores",
  tag="highscore-table",
  params(
    ("uuid" = OpenApiUuid, Path, description = "Highscore table UUID"),
  ),
  responses(
    (status = 200, description = "Highscore table details", body = ApiSuccessResponseBody<ScoresResponse>),
    (status = 403, description = "Forbidden"),
    (status = 404, description = "Highscore table not found"),
  ),
)]
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
  let scores = get_scores_for_table(highscore_table_id, None, &mut db).await?;
  Ok(ApiSuccessResponse::new(scores))
}

pub async fn get_scores_for_table(highscore_table_id: i32, limit: Option<u32>, db: &mut AsyncPgConnection) -> diesel::QueryResult<ScoresResponse> {
  let mut query = schema::highscore_table_entries::table
    .filter(schema::highscore_table_entries::highscore_table_id.eq(highscore_table_id))
    .order((schema::highscore_table_entries::player_score.desc(), schema::highscore_table_entries::creation_timestamp.asc()))
    .into_boxed();
  if let Some(limit) = limit {
    query = query.limit(limit as i64);
  }
  let entries = query
    .load::<models::HighscoreTableEntry>(db)
    .await?;
  let entries = entries.into_iter().map(ScoresResponseEntry::from).collect();
  Ok(ScoresResponse { scores: entries })
}
