
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
  ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetHighscoreTableParams {
  pub table_uuid: Uuid,
}

#[get("/scores", data = "<params>")]
async fn get_highscore_table_scores(params: DataFromStr<GameRequestPayload>, mut db: Connection<db::Db>) -> Result<ApiSuccessResponse<ScoresResponse>, ApiError> {
  let params = GameRequestBody::<GetHighscoreTableParams>::full_verify(&params, &mut db).await?;
  let highscore_table_id = schema::highscore_tables::table
    .filter(schema::highscore_tables::table_uuid.eq(params.body.table_uuid))
    .select(schema::highscore_tables::id)
    .first::<i32>(&mut db)
    .await?;
  let scores = get_scores_for_table(highscore_table_id, &mut db).await?;
  Ok(ApiSuccessResponse::new(scores))
}
