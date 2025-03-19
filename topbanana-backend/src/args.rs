
//! Command line argument parser.

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
  /// If supplied, seed the database with an initial admin user
  /// instead of running the Rocket server.
  #[arg(long)]
  pub generate_initial_user: bool,
  /// Force the command, even if dangerous.
  #[arg(long)]
  pub force: bool,
}
