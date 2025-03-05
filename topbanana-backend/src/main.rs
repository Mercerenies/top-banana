
use topbanana::server::run_server;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
  run_server().await?;
  Ok(())
}
