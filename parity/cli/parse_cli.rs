use structopt::StructOpt;

use cli::globals::Globals;
use cli::subcommands::SubCommands;

#[derive(StructOpt, Debug, Clone, Default)]
pub struct ArgsInput {
	#[structopt(subcommand)]
	pub subcommands: Option<SubCommands>,
	#[structopt(flatten)]
	pub globals: Globals,
}
