
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = super::schema::developers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Developer {
  pub id: i32,
  pub developer_uuid: Uuid,
  pub name: String,
  pub email: String,
  pub url: Option<String>,
  pub is_admin: bool,
  pub api_key: Option<String>,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = super::schema::developers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDeveloper {
  pub developer_uuid: Uuid,
  pub name: String,
  pub email: String,
  pub url: Option<String>,
  pub is_admin: bool,
  pub api_key: Option<String>,
}

#[derive(Queryable, Selectable, Associations, Clone)]
#[diesel(belongs_to(Developer))]
#[diesel(table_name = super::schema::games)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Game {
  pub id: i32,
  pub developer_id: i32,
  pub game_uuid: Uuid,
  pub game_secret_key: String,
  pub name: String,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = super::schema::games)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewGame {
  pub developer_id: i32,
  pub game_uuid: Uuid,
  pub game_secret_key: String,
  pub name: String,
}

#[derive(Queryable, Selectable, Associations, Clone)]
#[diesel(belongs_to(Game))]
#[diesel(table_name = super::schema::highscore_tables)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HighscoreTable {
  pub id: i32,
  pub game_id: i32,
  pub name: String,
  pub table_uuid: Uuid,
  pub maximum_scores_retained: Option<i32>,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = super::schema::highscore_tables)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHighscoreTable {
  pub game_id: i32,
  pub name: String,
  pub table_uuid: Uuid,
  pub maximum_scores_retained: Option<i32>,
}

#[derive(Queryable, Selectable, Associations, Clone)]
#[diesel(belongs_to(HighscoreTable))]
#[diesel(table_name = super::schema::highscore_table_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HighscoreTableEntry {
  pub id: i32,
  pub highscore_table_id: i32,
  pub player_name: String,
  pub player_score: f64,
  pub player_score_metadata: Option<String>,
  pub creation_timestamp: chrono::NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = super::schema::highscore_table_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHighscoreTableEntry {
  pub highscore_table_id: i32,
  pub player_name: String,
  pub player_score: f64,
  pub player_score_metadata: Option<String>,
}
