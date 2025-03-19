
use topbanana::server::run_server;
use topbanana::setup::{generate_initial_user, cleanup_historical_requests, setup_logger};
use topbanana::args::CliArgs;

use clap::Parser;

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
  let cli_args = CliArgs::parse();

  if cli_args.generate_initial_user {
    generate_initial_user(cli_args.force).await?;
  } else if cli_args.cleanup_historical_requests {
    cleanup_historical_requests().await?;
  } else {
    setup_logger()?;
    run_server().await?;
  }

  Ok(())
}
