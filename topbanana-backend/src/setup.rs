
use crate::db::models::NewDeveloper;
use crate::db::schema;
use crate::util::generate_key;

use fern::{Dispatch, InitError, log_file};
use humantime::format_rfc3339_seconds;
use log::LevelFilter;
use uuid::Uuid;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncPgConnection};
use chrono::{Duration, Utc};

use std::env;
use std::time::SystemTime;
use std::io::stdout;

pub async fn generate_initial_user(force: bool) -> anyhow::Result<()> {
  let mut connection = AsyncPgConnection::establish(&env::var("DATABASE_URL")?).await?;

  println!("Running initial admin user setup ...");

  let existing_admin_user = schema::developers::table
    .filter(schema::developers::is_admin.eq(true));
  if !force && diesel::select(diesel::dsl::exists(existing_admin_user)).get_result(&mut connection).await? {
    println!("Admin user already exists, refusing to create another.");
    println!("You may override this with --force if you know what you're doing.");
    return Ok(());
  }

  let developer_uuid = Uuid::new_v4();
  let api_key = generate_key();
  let new_developer = NewDeveloper {
    developer_uuid,
    name: String::from("System Administrator"),
    email: String::from("admin@example.com"),
    url: None,
    is_admin: true,
    api_key: Some(api_key),
  };
  diesel::insert_into(schema::developers::table)
    .values(&new_developer)
    .execute(&mut connection)
    .await?;

  println!("Successfully created admin user.");
  println!("  name = {}", new_developer.name);
  println!("  email = {}", new_developer.email);
  println!("  api key = {}", new_developer.api_key.unwrap());
  Ok(())
}

pub async fn cleanup_historical_requests() -> anyhow::Result<()> {
  let mut connection = AsyncPgConnection::establish(&env::var("DATABASE_URL")?).await?;

  println!("Cleaning up historical request records ...");

  let rows_to_delete = schema::historical_requests::table
    .filter(schema::historical_requests::timestamp.lt(Utc::now() - Duration::days(7)));
  let deleted_rows_count = diesel::delete(rows_to_delete)
    .execute(&mut connection)
    .await?;

  println!("Successfully deleted {} historical request record(s).", deleted_rows_count);
  Ok(())
}

/// Initialize the logger for this process.
pub fn setup_logger() -> Result<(), InitError> {
  Dispatch::new()
    .format(|out, message, record| {
      out.finish(format_args!(
        "[{} {} {}] {}",
        format_rfc3339_seconds(SystemTime::now()),
        record.level(),
        record.target(),
        message
      ))
    })
    .level(LevelFilter::Debug)
    .chain(log_file("log/output.log")?)
    .chain(
      Dispatch::new()
        .level(LevelFilter::Info)
        .chain(stdout())
    )
    .apply()?;
  Ok(())
}
