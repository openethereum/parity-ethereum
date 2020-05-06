use serde_derive::Deserialize;
use structopt::StructOpt;

#[derive(StructOpt, Deserialize, Debug, Clone)]
pub enum SubCommands {
	Daemon(Daemon),
	Wallet {
		#[structopt(subcommand)]
		wallet: Wallet,
	},
	Account {
		#[structopt(subcommand)]
		account: Account,
	},
	Import(Import),
	Export {
		#[structopt(subcommand)]
		export: Export,
	},
	Signer(Signer),
	Snapshots(Snapshots),
	Restore(Restore),
	Db(Db),
	#[structopt(
		about = "Print the hashed light clients headers of the given --chain (default: mainnet) in a JSON format. To be used as hardcoded headers in a genesis file."
	)]
	ExportHardcodedSync,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Use parity as a daemon")]
pub struct Daemon {
	#[structopt(long = "pid-file", name = "PID-FILE", help = "Path to the pid file")]
	pub pid_file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Manage accounts")]
pub enum Account {
	#[structopt(
		about = "Create a new account (and its associated key) for the given --chain [default: mainnet]"
	)]
	New,
	#[structopt(about = "List existing accounts of the given --chain [default: mainnet]")]
	List,
	#[structopt(
		about = "Import accounts from JSON UTC keystore files to the specified --chain [default: mainnet]"
	)]
	// FIXME: The original parser implementation had this as `Option<Vec<String>>` but this is not
	// supported by structopt yet, referring to issue
	// [#364](https://github.com/TeXitoi/structopt/issues/364)
	Import { path: Vec<String> },
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Manage wallet")]
pub enum Wallet {
	#[structopt(help = "Import wallet into the given chain (default: mainnet)")]
	Import {
		#[structopt(name = "PATH", help = "Path to the wallet")]
		path: Option<String>,
	},
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(
	about = "Import blockchain data from a file to the given chain database (default: mainnet)"
)]
pub struct Import {
	#[structopt(
		long,
		name = "FORMAT",
		help = "Import in a given format, FORMAT must be either 'hex' or 'binary'. (default: auto)"
	)]
	pub format: Option<String>,

	#[structopt(name = "FILE", long, help = "Path to the file to import from")]
	pub file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Export blockchain")]
pub enum Export {
	Blocks(ExportBlocks),
	State(ExportState),
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(
	about = "Export the blockchain blocks from the given chain database [default: mainnet] into a file. The command requires the chain to be synced with --fat-db on."
)]
pub struct ExportBlocks {
	#[structopt(
		long,
		name = "FORMAT",
		help = "Export in a given format. FORMAT must be 'hex' or 'binary'. [default: binary]"
	)]
	pub format: Option<String>,

	#[structopt(
		long,
		name = "FROM_BLOCK",
		help = "Export from block FROM_BLOCK, which may be an index or hash ",
		default_value = "1"
	)]
	pub from: String,

	#[structopt(
		long,
		name = "TO_BLOCK",
		help = "Export to (including TO_BLOCK) block TO_BLOCK, which may be an index, hash or 'latest'",
		default_value = "latest"
	)]
	pub to: String,
	#[structopt(help = "Path to the exported file", name = "FILE")]
	pub file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(
	about = "Export the blockchain state from the given chain [default: mainnet] into a file. The command requires the chain to be synced with --fat-db on."
)]
pub struct ExportState {
	#[structopt(long = "no-storage", help = "Don't export account storage.")]
	pub no_storage: bool,

	#[structopt(long = "no-code", help = "Don't export account code.")]
	pub no_code: bool,

	#[structopt(
		long = "max-balance",
		name = "MAX_WEI",
		help = "Don't export accounts with balance greater than specified."
	)]
	pub max_balance: Option<String>,

	#[structopt(
		long = "min-balance",
		name = "MIN_WEI",
		help = "Don't export accounts with balance less than specified."
	)]
	pub min_balance: Option<String>,

	#[structopt(
		default_value = "latest",
		long,
		name = "BLOCK",
		help = "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive"
	)]
	pub at: String,

	#[structopt(
		long,
		name = "FORMAT",
		help = "Export in a given format. FORMAT must be either 'hex' or 'binary'. [default: binary]"
	)]
	pub format: Option<String>,

	#[structopt(long = "file", name = "FILE", help = "Path to the exported file")]
	pub file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Manage Signer")]
pub enum Signer {
	#[structopt(
		about = "Generate a new signer-authentication token for the given --chain (default: mainnet)"
	)]
	NewToken,
	#[structopt(
		about = "List the signer-authentication tokens from given --chain (default: mainnet)"
	)]
	List,
	#[structopt(about = "Sign")]
	Sign {
		#[structopt(name = "ID")]
		id: Option<usize>,
	},
	#[structopt(about = "Reject")]
	Reject {
		#[structopt(name = "ID")]
		id: Option<usize>,
	},
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Make a snapshot of the database of the given chain (default: mainnet)")]
pub struct Snapshots {
	#[structopt(
		default_value = "latest",
		name = "BLOCK",
		help = "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive"
	)]
	pub at: String,

	#[structopt(name = "FILE", help = "Path to the file to export to")]
	pub file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(
	about = "Restore the databse of the given chain (default: mainnet) from a snapshot file"
)]
pub struct Restore {
	#[structopt(name = "FILE", help = "Path to the file to restore from")]
	pub file: Option<String>,
}

#[derive(StructOpt, Deserialize, Debug, Clone)]
#[structopt(about = "Manage the Database representing the state of the blockchain on this system")]
pub enum Db {
	#[structopt(about = "Clean the database of the given --chain (default: mainnet)")]
	Kill,
	#[structopt(about = "Removes NUM latests blocks from the db")]
	Reset {
		#[structopt(
			default_value = "10",
			name = "NUM",
			help = "Number of blocks to revert"
		)]
		num: u32,
	},
}
