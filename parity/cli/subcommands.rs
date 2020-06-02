use serde_derive::Deserialize;
use clap::Clap;

#[derive(Clap, Deserialize, Debug, Clone)]
pub enum SubCommands {
	Daemon(Daemon),
	Wallet {
		#[clap(subcommand)]
		wallet: Wallet,
	},
	Account {
		#[clap(subcommand)]
		account: Account,
	},
	Import(Import),
	Export {
		#[clap(subcommand)]
		export: Export,
	},
	Signer(Signer),
	Snapshots(Snapshots),
	Restore(Restore),
	Db(Db),
	#[clap(
		about = "Print the hashed light clients headers of the given --chain (default: mainnet) in a JSON format. To be used as hardcoded headers in a genesis file."
	)]
	ExportHardcodedSync,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Use parity as a daemon")]
pub struct Daemon {
	#[clap(long = "pid-file", name = "PID-FILE", about = "Path to the pid file")]
	pub pid_file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Manage accounts")]
pub enum Account {
	#[clap(
		about = "Create a new account (and its associated key) for the given --chain [default: mainnet]"
	)]
	New,
	#[clap(about = "List existing accounts of the given --chain [default: mainnet]")]
	List,
	#[clap(
		about = "Import accounts from JSON UTC keystore files to the specified --chain [default: mainnet]"
	)]
	// FIXME: The original parser implementation had this as `Option<Vec<String>>` but this is not
	// supported by structopt yet, referring to issue
	// [#364](https://github.com/TeXitoi/structopt/issues/364)
	Import { path: Vec<String> },
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Manage wallet")]
pub enum Wallet {
	#[clap(about = "Import wallet into the given chain (default: mainnet)")]
	Import {
		#[clap(name = "PATH", about = "Path to the wallet")]
		path: Option<String>,
	},
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(
	about = "Import blockchain data from a file to the given chain database (default: mainnet)"
)]
pub struct Import {
	#[clap(
		long,
		name = "FORMAT",
		about = "Import in a given format, FORMAT must be either 'hex' or 'binary'. (default: auto)"
	)]
	pub format: Option<String>,

	#[clap(name = "FILE", long, about = "Path to the file to import from")]
	pub file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Export blockchain")]
pub enum Export {
	Blocks(ExportBlocks),
	State(ExportState),
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(
	about = "Export the blockchain blocks from the given chain database [default: mainnet] into a file. The command requires the chain to be synced with --fat-db on."
)]
pub struct ExportBlocks {
	#[clap(
		long,
		name = "FORMAT",
		about = "Export in a given format. FORMAT must be 'hex' or 'binary'. [default: binary]"
	)]
	pub format: Option<String>,

	#[clap(
		long,
		name = "FROM_BLOCK",
		about = "Export from block FROM_BLOCK, which may be an index or hash ",
		default_value = "1"
	)]
	pub from: String,

	#[clap(
		long,
		name = "TO_BLOCK",
		about = "Export to (including TO_BLOCK) block TO_BLOCK, which may be an index, hash or 'latest'",
		default_value = "latest"
	)]
	pub to: String,
	#[clap(about = "Path to the exported file", name = "FILE")]
	pub file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(
	about = "Export the blockchain state from the given chain [default: mainnet] into a file. The command requires the chain to be synced with --fat-db on."
)]
pub struct ExportState {
	#[clap(long = "no-storage", about = "Don't export account storage.")]
	pub no_storage: bool,

	#[clap(long = "no-code", about = "Don't export account code.")]
	pub no_code: bool,

	#[clap(
		long = "max-balance",
		name = "MAX_WEI",
		about = "Don't export accounts with balance greater than specified."
	)]
	pub max_balance: Option<String>,

	#[clap(
		long = "min-balance",
		name = "MIN_WEI",
		about = "Don't export accounts with balance less than specified."
	)]
	pub min_balance: Option<String>,

	#[clap(
		default_value = "latest",
		long,
		name = "BLOCK",
		about = "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive"
	)]
	pub at: String,

	#[clap(
		long,
		name = "FORMAT",
		about = "Export in a given format. FORMAT must be either 'hex' or 'binary'. [default: binary]"
	)]
	pub format: Option<String>,

	#[clap(long = "file", name = "FILE", about = "Path to the exported file")]
	pub file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Manage Signer")]
pub enum Signer {
	#[clap(
		about = "Generate a new signer-authentication token for the given --chain (default: mainnet)"
	)]
	NewToken,
	#[clap(
		about = "List the signer-authentication tokens from given --chain (default: mainnet)"
	)]
	List,
	#[clap(about = "Sign")]
	Sign {
		#[clap(name = "ID")]
		id: Option<usize>,
	},
	#[clap(about = "Reject")]
	Reject {
		#[clap(name = "ID")]
		id: Option<usize>,
	},
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Make a snapshot of the database of the given chain (default: mainnet)")]
pub struct Snapshots {
	#[clap(
		default_value = "latest",
		name = "BLOCK",
		about = "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive"
	)]
	pub at: String,

	#[clap(name = "FILE", about = "Path to the file to export to")]
	pub file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(
	about = "Restore the databse of the given chain (default: mainnet) from a snapshot file"
)]
pub struct Restore {
	#[clap(name = "FILE", about = "Path to the file to restore from")]
	pub file: Option<String>,
}

#[derive(Clap, Deserialize, Debug, Clone)]
#[clap(about = "Manage the Database representing the state of the blockchain on this system")]
pub enum Db {
	#[clap(about = "Clean the database of the given --chain (default: mainnet)")]
	Kill,
	#[clap(about = "Removes NUM latests blocks from the db")]
	Reset {
		#[clap(
			default_value = "10",
			name = "REVERT_NUM",
			about = "Number of blocks to revert"
		)]
		num: u32,
	},
}
