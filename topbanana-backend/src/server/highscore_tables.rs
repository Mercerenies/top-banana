
use crate::db::schema;
use crate::server::requests::{GameRequestPayload, GameRequestBody};
use crate::util::DataFromStr;
use super::db;
use super::error::{ApiSuccessResponse, ApiError};
use super::api::{get_scores_for_table, ScoresResponse};

use rocket::{Route, get, routes};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub fn highscore_table_routes() -> Vec<Route> {
  routes![
    get_highscore_table_scores,
    get_highscore_table_scores_with_limit,
  ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetHighscoreTableParams {
  pub table_uuid: Uuid,
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
