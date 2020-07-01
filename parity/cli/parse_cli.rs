use clap::{AppSettings::*, Clap};
use cli::globals::Globals;
use cli::subcommands::SubCommands;

/// OpenEthereum
#[derive(Clap, Debug, Clone, Default)]
#[clap(
    name = "OpenEthereum",
    about = "Fast and feature-rich multi-network Ethereum client.",
    setting = DeriveDisplayOrder,
)]
pub struct ArgsInput {
	#[clap(subcommand)]
	pub subcommands: Option<SubCommands>,
	#[clap(flatten)]
	pub globals: Globals,
}
