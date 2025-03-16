
use crate::db::{schema, models};
use crate::server::requests::{GameRequestPayload, GameRequestBody};
use crate::util::DataFromStr;
use super::db;
use super::error::{ApiSuccessResponse, ApiError};
use super::api::{get_scores_for_table, ScoresResponse};

use rocket::{Route, get, post, routes};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncPgConnection};
use scoped_futures::ScopedFutureExt;

pub fn highscore_table_routes() -> Vec<Route> {
  routes![
    get_highscore_table_scores,
    get_highscore_table_scores_with_limit,
    post_new_highscore_table_score,
  ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetHighscoreTableParams {
  pub table_uuid: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostHighscoreTableParams {
  pub table_uuid: Uuid,
  pub player_name: String,
  pub player_score: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub player_score_metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PostHighscoreTableResponse {
  pub message: &'static str,
}

#[get("/scores", data = "<params>")]
async fn get_highscore_table_scores(
  params: DataFromStr<GameRequestPayload>,
  db: Connection<db::Db>,
) -> Result<ApiSuccessResponse<ScoresResponse>, ApiError> {
  get_highscore_table_scores_impl(params, None, db).await
}

#[get("/scores?<limit>", data = "<params>")]
async fn get_highscore_table_scores_with_limit(
  params: DataFromStr<GameRequestPayload>,
  limit: u32,
  db: Connection<db::Db>,
) -> Result<ApiSuccessResponse<ScoresResponse>, ApiError> {
  get_highscore_table_scores_impl(params, Some(limit), db).await
}

#[post("/scores/new", data = "<params>")]
async fn post_new_highscore_table_score(
  params: DataFromStr<GameRequestPayload>,
  mut db: Connection<db::Db>,
) -> Result<ApiSuccessResponse<PostHighscoreTableResponse>, ApiError> {
  let params = GameRequestBody::<PostHighscoreTableParams>::full_verify(&params, &mut db).await?;
  // Note: Filter on game UUID as well. If the user gives a mismatched
  // game UUID and table UUID, we have to reject the request for
  // security reasons.
  let (highscore_table_id, maximum_scores_retained) = schema::highscore_tables::table
    .inner_join(schema::games::table)
    .filter(schema::highscore_tables::table_uuid.eq(params.body.table_uuid))
    .filter(schema::games::game_uuid.eq(params.game_uuid))
    .select((schema::highscore_tables::id, schema::highscore_tables::maximum_scores_retained))
    .first::<(i32, Option<i32>)>(&mut db)
    .await?;
  let new_entry = models::NewHighscoreTableEntry {
    highscore_table_id,
    player_name: params.body.player_name,
    player_score: params.body.player_score,
    player_score_metadata: params.body.player_score_metadata,
  };

  db.transaction::<(), diesel::result::Error, _>(|db| async move {
    diesel::insert_into(schema::highscore_table_entries::table)
      .values(&new_entry)
      .execute(db)
      .await?;
    remove_extra_highscore_rows(highscore_table_id, maximum_scores_retained, db).await?;
    Ok(())
  }.scope_boxed()).await?;

  let resp = PostHighscoreTableResponse { message: "New score added successfully" };
  Ok(ApiSuccessResponse::new(resp))
}

async fn get_highscore_table_scores_impl(
  params: DataFromStr<GameRequestPayload>,
  limit: Option<u32>,
  mut db: Connection<db::Db>,
) -> Result<ApiSuccessResponse<ScoresResponse>, ApiError> {
  let params = GameRequestBody::<GetHighscoreTableParams>::full_verify(&params, &mut db).await?;
  // Note: Filter on game UUID as well. If the user gives a mismatched
  // game UUID and table UUID, we have to reject the request for
  // security reasons.
  let highscore_table_id = schema::highscore_tables::table
    .inner_join(schema::games::table)
    .filter(schema::highscore_tables::table_uuid.eq(params.body.table_uuid))
    .filter(schema::games::game_uuid.eq(params.game_uuid))
    .select(schema::highscore_tables::id)
    .first::<i32>(&mut db)
    .await?;
  let scores = get_scores_for_table(highscore_table_id, limit, &mut db).await?;
  Ok(ApiSuccessResponse::new(scores))
}

async fn remove_extra_highscore_rows(
  table_id: i32,
  maximum_scores_retained: Option<i32>,
  db: &mut AsyncPgConnection,
) -> diesel::QueryResult<()> {
  use schema::highscore_table_entries::dsl::*;

  let Some(maximum_scores_retained) = maximum_scores_retained else {
    // Nothing to do.
    return Ok(())
  };

  let retained_entries = diesel::alias!(schema::highscore_table_entries as retained_entries);

  let scores_to_retain = retained_entries
    .filter(retained_entries.field(highscore_table_id).eq(table_id))
    .order((retained_entries.field(player_score).desc(), retained_entries.field(creation_timestamp).asc()))
    .limit(maximum_scores_retained as i64)
    .select(retained_entries.field(id));
  diesel::delete(highscore_table_entries)
    .filter(id.ne_all(scores_to_retain))
    .execute(db)
    .await?;
  Ok(())
}
