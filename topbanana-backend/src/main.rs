
use topbanana::server::run_server;
use topbanana::setup::generate_initial_user;
use topbanana::args::CliArgs;

use clap::Parser;

#[rocket::main]
async fn main() -> Result<(), anyhow::Error> {
  let cli_args = CliArgs::parse();

  if cli_args.generate_initial_user {
    generate_initial_user(cli_args.force).await?;
  } else {
    run_server().await?;
  }

  Ok(())
}
