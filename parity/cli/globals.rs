use serde_derive::{Deserialize, Serialize};
use clap::Clap;

const OPERATING: &str = "Operating Options";
const CONVENIENCE: &str = "Convenience Options";
const ACCOUNT: &str = "Account Options";
const PRIVATE_TRANSACTIONS: &str = "Private Transactions Options";
const UI: &str = "UI Options";
const NETWORKING: &str = "Nethworking Options";
const IPC: &str = "IPC Options";
const HTTP_JSON_RPC: &str = "HTTP/JSONRPC Options";
const LIGHT_CLIENT: &str = "Light Client Options";
const WEBSOCKETS: &str = "WebSockets Options";
const SECRET_STORE: &str = "Secret Store Options";
const SEALING_MINING: &str = "Sealing/Mining Options";
const INTERNAL: &str = "Internal Options";
const MISCELLANEOUS: &str = "Miscellaneous Options";
const FOOTPRINT: &str = "Footprint Options";
const IMPORT_EXPORT: &str = "Import/Export Options";
const SNAPSHOT: &str = "Snapshot Options";
const LEGACY: &str = "Legacy Options";

#[derive(PartialEq, Default, Clap, Serialize, Deserialize, Debug, Clone)]
pub struct Globals {
	#[clap(flatten)]
	pub operating: OperatingOptions,

	#[clap(flatten)]
	pub convenience: ConvenienceOptions,

	#[clap(flatten)]
	pub account: AccountOptions,

	#[clap(flatten)]
	pub private_transactions: PrivateTransactions,

	#[clap(flatten)]
	pub ui_options: UIOptons,

	#[clap(flatten)]
	pub networking: NetworkingOptions,

	#[clap(flatten)]
	pub ipc: IPCOptions,

	#[clap(flatten)]
	pub http_json_rpc: HttpJsonRpcOptions,

	#[clap(flatten)]
	pub light_client: LightClientOptions,

	#[clap(flatten)]
	pub websockets: WebsocketsOptions,

	#[clap(flatten)]
	pub secret_store: SecretStoreOptions,

	#[clap(flatten)]
	pub sealing_mining: SealingMiningOptions,

	#[clap(flatten)]
	pub internal: InternalOptions,

	#[clap(flatten)]
	pub miscellaneous: MiscellaneousOptions,

	#[clap(flatten)]
	pub footprint: FootPrintOptions,

	#[clap(flatten)]
	pub import_export: ImportExportOptions,

	#[clap(flatten)]
	pub snapshot: SnapshotOptions,

	#[clap(flatten)]
	pub legacy: LegacyOptions,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct OperatingOptions {
	#[clap(
		long = "no-download",
		about = "Normally new releases will be downloaded ready for updating. This disables it. Not recommended.",
		help_heading = Some(OPERATING),
	)]
	pub no_download: bool,

	#[clap(
		long = "no-consensus",
		about = "Force the binary to run even if there are known issues regarding consensus. Not recommended.",
		help_heading = Some(OPERATING),
	)]
	pub no_consensus: bool,

	#[clap(
		long,
		about = "Experimental: run in light client mode. Light clients synchronize a bare minimum of data and fetch necessary data on-demand from the network. Much lower in storage, potentially higher in bandwidth. Has no effect with subcommands.",
		help_heading = Some(OPERATING),
	)]
	pub light: bool,

	#[clap(
		long = "no-hardcoded-sync",
		about = "By default, if there is no existing database the light client will automatically jump to a block hardcoded in the chain's specifications. This disables this feature.",
		help_heading = Some(OPERATING),
	)]
	pub no_hardcoded_sync: bool,

	#[clap(
		long = "force-direct",
		about = "Run the originally installed version of Parity, ignoring any updates that have since been installed.",
		help_heading = Some(OPERATING),
	)]
	pub force_direct: bool,

	#[clap(
		name = "MODE",
		long,
		about = "Set the operating mode. MODE can be one of: last - Uses the last-used mode, active if none; active - Parity continuously syncs the chain; passive - Parity syncs initially, then sleeps and wakes regularly to resync; dark - Parity syncs only when the JSON-RPC is active; offline - Parity doesn't sync.",
		help_heading = Some(OPERATING),
	)]
	pub mode: Option<String>,

	#[clap(
		long = "mode-timeout",
		name = "TIMEOUT_IN_SECS",
		about = "Specify the number of seconds before inactivity timeout occurs when mode is dark or passive",
		help_heading = Some(OPERATING),
	)]
	pub mode_timeout: Option<u64>,

	#[clap(
		long = "mode-alarm",
		name = "ALARM_IN_SECS",
		about = "Specify the number of seconds before auto sleep reawake timeout occurs when mode is passive",
		help_heading = Some(OPERATING),
	)]
	pub mode_alarm: Option<u64>,

	#[clap(
		long = "auto-update",
		name = "SET",
		about = "Set a releases set to automatically update and install. SET can be one of: all - All updates in the our release track; critical - Only consensus/security updates; none - No updates will be auto-installed.",
		help_heading = Some(OPERATING),
	)]
	pub auto_update: Option<String>,

	#[clap(
		long = "auto-update-delay",
		name = "DELAY_NUM",
		about = "Specify the maximum number of blocks used for randomly delaying updates.",
		help_heading = Some(OPERATING),
	)]
	pub auto_update_delay: Option<u16>,

	#[clap(
		long = "auto-update-check-frequency",
		name = "FREQUENCY_NUM",
		about = "Specify the number of blocks between each auto-update check.",
		help_heading = Some(OPERATING),
	)]
	pub auto_update_check_frequency: Option<u16>,

	#[clap(
		long = "release-track",
		name = "TRACK",
		about = "Set which release track we should use for updates. TRACK can be one of: stable - Stable releases; nightly - Nightly releases (unstable); testing - Testing releases (do not use); current - Whatever track this executable was released on.",
		help_heading = Some(OPERATING),
	)]
	pub release_track: Option<String>,

	#[clap(
		long,
		name = "CHAIN",
		about = "Specify the blockchain type. CHAIN may be either a JSON chain specification file or ethereum, classic, classic-no-phoenix, poacore, xdai, volta, ewc, musicoin, ellaism, mix, callisto, ethercore, mordor, ropsten, kovan, rinkeby, goerli, kotti, poasokol, testnet, evantestcore, evancore or dev.",
		help_heading = Some(OPERATING),
	)]
	pub chain: Option<String>,

	#[clap(
		long = "keys-path",
		name = "KEYS_PATH",
		about = "Specify the path for JSON key files to be found",
		help_heading = Some(OPERATING),
	)]
	pub keys_path: Option<String>,

	#[clap(
		name = "NAME",
		long,
		about = "Specify your node's name.",
		help_heading = Some(OPERATING),
	)]
	pub identity: Option<String>,

	#[clap(
		short = "d",
		long = "base-path",
		name = "BASE_PATH",
		about = "Specify the base data storage path.",
		help_heading = Some(OPERATING),
	)]
	pub base_path: Option<String>,

	#[clap(
		long = "db-path",
		name = "DB_PATH",
		about = "Specify the database directory path",
		help_heading = Some(OPERATING),
	)]
	pub db_path: Option<String>,

	#[clap(
		long = "sync-until",
		name = "BLOCK_TO_SYNC_UNTIL",
		about = "Sync until the given block has been imported, then enter offline mode. Intended for debug/benchmarking only.",
		help_heading = Some(OPERATING),
	)]
	pub sync_until: Option<u64>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ConvenienceOptions {
	#[clap(
		long = "unsafe-expose",
		about = "All servers will listen on external interfaces and will be remotely accessible. It's equivalent with setting the following: --[ws,jsonrpc,secretstore,stratum,dapps,secretstore-http]-interface=all --*-hosts=all	 This option is UNSAFE and should be used with great care!",
		help_heading = Some(CONVENIENCE),
	)]
	pub unsafe_expose: bool,

	#[clap(
		short,
		long,
		name = "CONFIG",
		about = "Specify a configuration. CONFIG may be either a configuration file or a preset: dev, insecure, dev-insecure, mining, or non-standard-ports.",
		help_heading = Some(CONVENIENCE),
	)]
	pub config: Option<String>,

	#[clap(
		long = "config-generate",
		name = "PATH_TO_GENERATE_CONFIG_IN",
		about = "Save the current flags and their values into a configuration for future use",
		help_heading = Some(CONVENIENCE),
	)]
	pub config_generate: Option<String>,

	#[clap(
		long = "ports-shift",
		name = "SHIFT",
		about = "Add SHIFT to all port numbers Parity is listening on. Includes network port and all servers (HTTP JSON-RPC, WebSockets JSON-RPC, SecretStore).",
		help_heading = Some(CONVENIENCE),
	)]
	pub ports_shift: Option<u16>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AccountOptions {
	#[clap(
		long = "fast-unlock",
		name = "FAST_UNLOCK_BOOL",
		about = "Use drastically faster unlocking mode. This setting causes raw secrets to be stored unprotected in memory, so use with care.",
		help_heading = Some(ACCOUNT),
	)]
	pub fast_unlock: bool,

	#[clap(
		long = "keys-iterations",
		name = "NUM_KEYS_INTERATIONS",
		about = "Specify the number of iterations to use when deriving key from the password (bigger is more secure)",
		help_heading = Some(ACCOUNT),
	)]
	pub keys_iterations: Option<u32>,

	#[clap(
		long = "accounts-refresh",
		name = "TIME",
		about = "Specify the cache time of accounts read from disk. If you manage thousands of accounts set this to 0 to disable refresh.",
		help_heading = Some(ACCOUNT),
	)]
	pub accounts_refresh: Option<u64>,

	#[clap(
		long,
		name = "UNLOCK_ACCOUNTS",
		about = "Unlock UNLOCK_ACCOUNTS for the duration of the execution. UNLOCK_ACCOUNTS is a comma-delimited list of addresses.",
		help_heading = Some(ACCOUNT),
	)]
	pub unlock: Option<String>,

	#[clap(
		long = "enable-signing-queue",
		name = "BOOLEAN",
		about = "Enables the signing queue for external transaction signing either via CLI or personal_unlockAccount, turned off by default.",
		help_heading = Some(ACCOUNT),
	)]
	pub enable_signing_queue: bool,

	#[clap(
		long,
		name = "FILE",
		about = "Provide a file containing a password for unlocking an account. Leading and trailing whitespace is trimmed.",
		help_heading = Some(ACCOUNT),
	)]
	pub password: Vec<String>, // FIXME: Why is this a Vec?
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct PrivateTransactions {
	#[clap(
		long = "private-tx-enabled",
		about = "Enable private transactions.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_enabled: bool,

	#[clap(
		long = "private-state-offchain",
		about = "Store private state offchain (in the local DB).",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_state_offchain: bool,

	#[clap(
		long = "private-signer",
		name = "ACCOUNT",
		about = "Specify the account for signing public transaction created upon verified private transaction.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_signer: Option<String>,

	#[clap(
		long = "private-validators",
		name = "ACCOUNTS",
		about = "Specify the accounts for validating private transactions. ACCOUNTS is a comma-delimited list of addresses.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_validators: Option<String>,

	#[clap(
		long = "private-account",
		name = "PRIVATE_ACCOUNT",
		about = "Specify the account for signing requests to secret store.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_account: Option<String>,

	#[clap(
		long = "private-sstore-url",
		name = "URL",
		about = "Specify secret store URL used for encrypting private transactions.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_sstore_url: Option<String>,

	#[clap(
		long = "private-sstore-threshold",
		name = "THRESHOLD_NUM",
		about = "Specify secret store threshold used for encrypting private transactions.",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_sstore_threshold: Option<u32>,

	#[clap(
		long = "private-passwords",
		name = "PASS_FILE",
		about = "Provide a file containing passwords for unlocking accounts (signer, private account, validators).",
		help_heading = Some(PRIVATE_TRANSACTIONS),
	)]
	pub private_passwords: Option<String>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct UIOptons {
	#[clap(
		long = "ui-path",
		about = "Specify directory where Trusted UIs tokens should be stored.",
		help_heading = Some(UI),
	)]
	pub ui_path: Option<String>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct NetworkingOptions {
	#[clap(
		long = "no-warp",
		about = "Disable syncing from the snapshot over the network.",
		help_heading = Some(NETWORKING),
	)]
	pub no_warp: bool,

	#[clap(
		long = "no-discovery",
		about = "Disable new peer discovery.",
		help_heading = Some(NETWORKING),
	)]
	pub no_discovery: bool,

	#[clap(
		long = "reserved-only",
		about = "Connect only to reserved nodes.",
		help_heading = Some(NETWORKING),
	)]
	pub reserved_only: bool,

	#[clap(
		long = "no-ancient-blocks",
		about = "Disable downloading old blocks after snapshot restoration or warp sync. Not recommended.",
		help_heading = Some(NETWORKING),
	)]
	pub no_ancient_blocks: bool,

	#[clap(
		long = "no-serve-light",
		about = "Disable serving of light peers.",
		help_heading = Some(NETWORKING),
	)]
	pub no_serve_light: bool,

	#[clap(
		long = "warp-barrier",
		name = "WARP_BARRIER_NUM",
		about = "When warp enabled never attempt regular sync before warping to block WARP_BARRIER_NUM.",
		help_heading = Some(NETWORKING),
	)]
	pub warp_barrier: Option<u64>,

	#[clap(
		long,
		name = "PORT",
		about = "Override the port on which the node should listen.",
		help_heading = Some(NETWORKING),
	)]
	pub port: Option<u16>,

	#[clap(
		long,
		name = "IP",
		about = "Network interfaces. Valid values are 'all', 'local' or the ip of the interface you want parity to listen to.",
		help_heading = Some(NETWORKING),
	)]
	pub interface: Option<String>,

	#[clap(
		long = "min-peers",
		name = "MIN_NUM",
		about = "Try to maintain at least MIN_NUM peers.",
		help_heading = Some(NETWORKING),
	)]
	pub min_peers: Option<u16>,

	#[clap(
		long = "max-peers",
		name = "MAX_NUM",
		about = "Try to maintain at least MAX_NUM peers.",
		help_heading = Some(NETWORKING),
	)]
	pub max_peers: Option<u16>,

	#[clap(
		long = "snapshot-peers",
		name = "SNAPSHOT_NUM",
		about = "Allow additional SNAPSHOT_NUM peers for a snapshot sync.",
		help_heading = Some(NETWORKING),
	)]
	pub snapshot_peers: Option<u16>,

	#[clap(
		long,
		name = "METHOD",
		about = "Specify method to use for determining public address. Must be one of: any, none, upnp, extip:<IP>.",
		help_heading = Some(NETWORKING),
	)]
	pub nat: Option<String>,

	#[clap(
		long = "allow-ips",
		name = "FILTER",
		about = "Filter outbound connections. Must be one of: private - connect to private network IP addresses only; public - connect to public network IP addresses only; all - connect to any IP address.",
		help_heading = Some(NETWORKING),
	)]
	pub allow_ips: Option<String>,

	#[clap(
		long = "max-pending-peers",
		name = "PENDING_NUM",
		about = "Allow up to PENDING_NUM pending connections.",
		help_heading = Some(NETWORKING),
	)]
	pub max_pending_peers: Option<u16>,

	#[clap(
		long = "network-id",
		name = "INDEX",
		about = "Override the network identifier from the chain we are on.",
		help_heading = Some(NETWORKING),
	)]
	pub network_id: Option<u64>,

	#[clap(
		long,
		name = "BOOTNODES",
		about = "Override the bootnodes from our chain. NODES should be comma-delimited enodes.",
		help_heading = Some(NETWORKING),
	)]
	pub bootnodes: Option<String>,

	#[clap(
		long = "node-key",
		name = "NODE_KEY",
		about = "Specify node secret key, either as 64-character hex string or input to SHA3 operation.",
		help_heading = Some(NETWORKING),
	)]
	pub node_key: Option<String>,

	#[clap(
		long = "reserved-peers",
		name = "RESERVED_PEERS_FILE",
		about = "Provide a file containing enodes, one per line. These nodes will always have a reserved slot on top of the normal maximum peers.",
		help_heading = Some(NETWORKING),
	)]
	pub reserved_peers: Option<String>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct IPCOptions {
	#[clap(
		long = "no-ipc",
		about = "Provide a file containing enodes, one per line. These nodes will always have a reserved slot on top of the normal maximum peers.",
		help_heading = Some(IPC),
	)]
	pub no_ipc: bool,

	#[clap(
		long = "ipc-path",
		name = "IPC_PATH",
		about = "Provide a file containing enodes, one per line. These nodes will always have a reserved slot on top of the normal maximum peers.",
		help_heading = Some(IPC),
	)]
	pub ipc_path: Option<String>,

	#[clap(
		long = "ipc-chmod",
		name = "IPC_CHMOD_NUM",
		about = "Specify octal value for ipc socket permissions (unix/bsd only)",
		help_heading = Some(IPC),
	)]
	pub ipc_chmod: Option<String>,

	#[clap(
		long = "ipc-apis",
		name = "IPC_APIS",
		about = "Specify custom API set available via JSON-RPC over IPC using a comma-delimited list of API names. Possible names are: all, safe, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, rpc, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces, rpc",
		help_heading = Some(IPC),
	)]
	pub ipc_apis: Option<String>,
}

impl IPCOptions {
	pub fn ipc_path_default() -> String {
		if cfg!(windows) {
			r"\\.\pipe\jsonrpc.ipc".to_owned()
		} else {
			"$BASE/jsonrpc.ipc".to_owned()
		}
	}
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct HttpJsonRpcOptions {
	#[clap(
		long = "json-rpc-allow-missing-blocks",
		about = "RPC calls will return 'null' instead of an error if ancient block sync is still in progress and the block information requested could not be found",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_allow_missing_blocks: bool,

	#[clap(
		long = "no-jsonrpc",
		about = "Disable the HTTP JSON-RPC API server.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub no_jsonrpc: bool,

	#[clap(
		long = "jsonrpc-no-keep-alive",
		about = "Disable HTTP/1.1 keep alive header. Disabling keep alive will prevent re-using the same TCP connection to fire multiple requests, recommended when using one request per connection.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_no_keep_alive: bool,

	#[clap(
		long = "jsonrpc-experimental",
		about = "Enable experimental RPCs. Enable to have access to methods from unfinalised EIPs in all namespaces",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_experimental: bool,

	#[clap(
		long = "jsonrpc-port",
		name = "JSONRPC_PORT",
		about = "Specify the port portion of the HTTP JSON-RPC API server.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_port: Option<u16>,

	#[clap(
		long = "jsonrpc-interface",
		name = "JSONRPC_IP",
		about = "Specify the hostname portion of the HTTP JSON-RPC API server, JSONRPC_IP should be an interface's IP address, or all (all interfaces) or local.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_interface: Option<String>,

	#[clap(
		long = "jsonrpc-apis",
		name = "JSONRPC_APIS",
		about = "Specify the APIs available through the HTTP JSON-RPC interface using a comma-delimited list of API names. Possible names are: all, safe, debug, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, rpc, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces, rpc",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_apis: Option<String>,

	#[clap(
		long = "jsonrpc-hosts",
		name = "JSONRPC_HOSTS",
		about = "List of allowed Host header values. This option will validate the Host header sent by the browser, it is additional security against some attack vectors. Special options: \"all\", \"none\",.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_hosts: Option<String>,

	#[clap(
		long = "jsonrpc-server-threads",
		name = "JSONRPC_SERVER_THREADS",
		about = "Enables multiple threads handling incoming connections for HTTP JSON-RPC server.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_server_threads: Option<usize>,

	#[clap(
		name = "JSONRPC_CORS_URL",
		long = "jsonrpc-cors",
		about = "Specify CORS header for HTTP JSON-RPC API responses. Special options: \"all\", \"none\".",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_cors: Option<String>,

	#[clap(
		long = "jsonrpc-max-payload",
		name = "JSONRPC_MAX_MB",
		about = "Specify maximum size for HTTP JSON-RPC requests in megabytes.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub jsonrpc_max_payload: Option<usize>,

	#[clap(
		name = "POLL_LIFETIME_SECS",
		long = "poll-lifetime",
		about = "Set the RPC filter lifetime to S seconds. The filter has to be polled at least every S seconds , otherwise it is removed.",
		help_heading = Some(HTTP_JSON_RPC),
	)]
	pub poll_lifetime: Option<u32>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct LightClientOptions {
	#[clap(
		long = "on-demand-time-window",
		about = "Specify the maximum time to wait for a successful response",
		name = "RESPONSE_SECS",
		help_heading = Some(LIGHT_CLIENT),
	)]
	pub on_demand_response_time_window: Option<u64>,

	#[clap(
		long = "on-demand-start-backoff",
		name = "BACKOFF_START_SECS",
		about = "Specify light client initial backoff time for a request",
		help_heading = Some(LIGHT_CLIENT),
	)]
	pub on_demand_request_backoff_start: Option<u64>,

	#[clap(
		long = "on-demand-end-backoff",
		name = "BACKOFF_END_SECS",
		about = "Specify light client maximam backoff time for a request",
		help_heading = Some(LIGHT_CLIENT),
	)]
	pub on_demand_request_backoff_max: Option<u64>,

	#[clap(
		long = "on-demand-max-backoff-rounds",
		name = "BACKOFF_MAX_ROUNDS_TIMES",
		about = "Specify light client maximam number of backoff iterations for a request",
		help_heading = Some(LIGHT_CLIENT),
	)]
	pub on_demand_request_backoff_rounds_max: Option<usize>,

	#[clap(
		long = "on-demand-consecutive-failures",
		name = "MAX_CONSECUTIVE_FAILURE_TIMES",
		about = "Specify light client the number of failures for a request until it gets exponentially backed off",
		help_heading = Some(LIGHT_CLIENT),
	)]
	pub on_demand_request_consecutive_failures: Option<usize>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct WebsocketsOptions {
	#[clap(
		about = "Disable the WebSockets JSON-RPC server.",
		long = "no-ws",
		help_heading = Some(WEBSOCKETS),
	)]
	pub no_ws: bool,

	#[clap(
		long = "ws-port",
		name = "WS_PORT",
		about = "Specify the port portion of the WebSockets JSON-RPC server.",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_port: Option<u16>,

	#[clap(
		long = "ws-interface",
		name = "WS_INTERFACE_IP",
		about = "Specify the hostname portion of the WebSockets JSON-RPC server, IP should be an interface's IP address, or all (all interfaces) or local.",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_interface: Option<String>,

	#[clap(
		long = "ws-apis",
		name = "WS_APIS",
		about = "Specify the JSON-RPC APIs available through the WebSockets interface using a comma-delimited list of API names. Possible names are: all, safe, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, rpc, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces, rpc",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_apis: Option<String>,

	#[clap(
		long = "ws-origins",
		name = "WS_ORIGINS_URL",
		about = "Specify Origin header values allowed to connect. Special options: \"all\", \"none\".",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_origins: Option<String>,

	#[clap(
		long = "ws-hosts",
		name = "WS_HOSTS",
		about = "List of allowed Host header values. This option will validate the Host header sent by the browser, it is additional security against some attack vectors. Special options: \"all\", \"none\".",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_hosts: Option<String>,

	#[clap(
		long = "ws-connections",
		name = "WS_MAX_CONN",
		about = "Maximum number of allowed concurrent WebSockets JSON-RPC connections.",
		help_heading = Some(WEBSOCKETS),
	)]
	pub ws_max_connections: Option<usize>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SecretStoreOptions {
	#[clap(
		long = "no-secretstore",
		about = "Disable Secret Store functionality.",
		help_heading = Some(SECRET_STORE),
	)]
	pub no_secretstore: bool,

	#[clap(
		long = "no-secretstore-http",
		about = "Disable Secret Store HTTP API.",
		help_heading = Some(SECRET_STORE),
	)]
	pub no_secretstore_http: bool,

	#[clap(
		long = "no-secretstore-auto-migrate",
		about = "Do not run servers set change session automatically when servers set changes. This option has no effect when servers set is read from configuration file.",
		help_heading = Some(SECRET_STORE),
	)]
	pub no_secretstore_auto_migrate: bool,

	#[clap(
		long = "secretstore-http-cors",
		name = "HTTP_CORS_URLS",
		about = "Specify CORS header for Secret Store HTTP API responses. Special options: \"all\", \"none\".",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_http_cors: Option<String>,

	#[clap(
		long = "secretstore-acl-contract",
		about = "Secret Store permissioning contract address source: none, registry (contract address is read from 'secretstore_acl_checker' entry in registry) or address.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_acl_contract: Option<String>,

	#[clap(
		long = "secrestore-contract",
		name = "SECRETSTORE_SOURCE",
		about = "Secret Store Service contract address source: none, registry (contract address is read from 'secretstore_service' entry in registry) or address.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_contract: Option<String>,

	#[clap(
		long = "secretstore-srv-gen-contract",
		name = "GEN_SOURCE",
		about = "Secret Store Service server key generation contract address source: none, registry (contract address is read from 'secretstore_service_srv_gen' entry in registry) or address.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_srv_gen_contract: Option<String>,

	#[clap(
		about = "Secret Store Service server key retrieval contract address source: none, registry (contract address is read from 'secretstore_service_srv_retr' entry in registry) or address.",
		name = "RETR_SOURCE",
		long = "secretstore-srv-retr-contract",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_srv_retr_contract: Option<String>,

	#[clap(
		about = "Secret Store Service document key store contract address source: none, registry (contract address is read from 'secretstore_service_doc_store' entry in registry) or address.",
		name = "DOC_STORE_SOURCE",
		long = "secretstore-doc-store-contract",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_doc_store_contract: Option<String>,

	#[clap(
		about = "Secret Store Service document key shadow retrieval contract address source: none, registry (contract address is read from 'secretstore_service_doc_sretr' entry in registry) or address.",
		name = "DOC_SRETR_SOURCE",
		long = "secretstore-doc-sretr-contract",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_doc_sretr_contract: Option<String>,

	#[clap(
		about = "Comma-separated list of other secret store cluster nodes in form NODE_PUBLIC_KEY_IN_HEX@NODE_IP_ADDR:NODE_PORT.",
		name = "SECRETSTORE_NODES",
		long = "secretstore-nodes",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_nodes: Option<String>,

	#[clap(
		name = "SET_CONTRACT_SOURCE",
		long = "secretstore-server-set-contract",
		about = "Secret Store server set contract address source: none, registry (contract address is read from 'secretstore_server_set' entry in registry) or address.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_server_set_contract: Option<String>,

	#[clap(
		long = "secretstore-interface-ip",
		name = "SECRETSTORE_IP",
		about = "Specify the hostname portion for listening to Secret Store Key Server internal requests, IP should be an interface's IP address, or local.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_interface: Option<String>,

	#[clap(
		long = "secretstore-port",
		name = "SECRETSTORE_PORT",
		about = "Specify the port portion for listening to Secret Store Key Server internal requests.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_port: Option<u16>,

	#[clap(
		long = "secretstore-http-interface",
		name = "SECRETSTORE_HTTP_INTERFACE",
		about = "Specify the hostname portion for listening to Secret Store Key Server HTTP requests, IP should be an interface's IP address, or local.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_http_interface: Option<String>,

	#[clap(
		long = "secretstore-http-port",
		name = "SECRETSTORE_HTTP_PORT",
		about = "Specify the port portion for listening to Secret Store Key Server HTTP requests.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_http_port: Option<u16>,

	#[clap(
		name = "SECRETSTORE_PATH",
		long = "secretstore-path",
		about = "Specify directory where Secret Store should save its data.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_path: Option<String>,

	#[clap(
		long = "secretstore-secret",
		name = "SECRETSTORE_SECRET",
		about = "Hex-encoded secret key of this node.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_secret: Option<String>,

	#[clap(
		long = "secretstore-admin-public",
		name = "SECRETSTORE_ADMIN_PUBLIC",
		about = "Hex-encoded public key of secret store administrator.",
		help_heading = Some(SECRET_STORE),
	)]
	pub secretstore_admin_public: Option<String>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SealingMiningOptions {
	#[clap(
		about = "Force the node to author new blocks as if it were always sealing/mining.",
		long = "force-sealing",
		help_heading = Some(SEALING_MINING),
	)]
	pub force_sealing: bool,

	#[clap(
		about = "Force the node to author new blocks when a new uncle block is imported.",
		long = "reseal-on-uncle",
		help_heading = Some(SEALING_MINING),
	)]
	pub reseal_on_uncle: bool,

	#[clap(
		about = "Move solved blocks from the work package queue instead of cloning them. This gives a slightly faster import speed, but means that extra solutions submitted for the same work package will go unused.",
		long = "remove-solved",
		help_heading = Some(SEALING_MINING),
	)]
	pub remove_solved: bool,

	#[clap(
		about = "Local transactions sent through JSON-RPC (HTTP, WebSockets, etc) will be treated as 'external' if the sending account is unknown.",
		long = "tx-queue-no-unfamiliar-locals",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_no_unfamiliar_locals: bool,

	#[clap(
		about = "Disables transaction queue optimization to early reject transactions below minimal effective gas price. This allows local transactions to always enter the pool, despite it being full, but requires additional ecrecover on every transaction.",
		long = "tx-queue-no-early-reject",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_no_early_reject: bool,

	#[clap(
		about = "Always refuse service transactions.",
		long = "refuse-service-transactions",
		help_heading = Some(SEALING_MINING),
	)]
	pub refuse_service_transactions: bool,

	#[clap(
		about = "Pending block will be created with maximal possible gas limit and will execute all transactions in the queue. Note that such block is invalid and should never be attempted to be mined.",
		long = "infinite-pending-block",
		help_heading = Some(SEALING_MINING),
	)]
	pub infinite_pending_block: bool,

	#[clap(
		about = "Don't save pending local transactions to disk to be restored whenever the node restarts.",
		long = "no-persistent-txqueue",
		help_heading = Some(SEALING_MINING),
	)]
	pub no_persistent_txqueue: bool,

	// For backward compatibility; Stratum should be enabled if the config file
	// contains a `[stratum]` section and it is not explicitly disabled (disable = true)
	#[clap(
		long,
		about = "Run Stratum server for miner push notification.",
		help_heading = Some(SEALING_MINING),
	)]
	pub stratum: bool,

	#[clap(
		long = "reseal-on-txs",
		name = "RESEAL_TXS_SET",
		about = "Specify which transactions should force the node to reseal a block. SET is one of: none - never reseal on new transactions; own - reseal only on a new local transaction; ext - reseal only on a new external transaction; all - reseal on all new transactions.",
		help_heading = Some(SEALING_MINING),
	)]
	pub reseal_on_txs: Option<String>,

	#[clap(
		long = "reseal-min-period",
		name = "RESEAL_MIN_MS",
		about = "Specify the minimum time between reseals from incoming transactions. MS is time measured in milliseconds.",
		help_heading = Some(SEALING_MINING),
	)]
	pub reseal_min_period: Option<u64>,

	#[clap(
		long = "reseal-max-period",
		name = "RESEAL_MAX_MS",
		about = "Specify the maximum time between reseals from incoming transactions. MS is time measured in milliseconds.",
		help_heading = Some(SEALING_MINING),
	)]
	pub reseal_max_period: Option<u64>,

	#[clap(
		name = "WORK_QUEUE_SIZE_ITEMS",
		long = "work-queue-size",
		about = "Specify the number of historical work packages which are kept cached lest a solution is found for them later. High values take more memory but result in fewer unusable solutions.",
		help_heading = Some(SEALING_MINING),
	)]
	pub work_queue_size: Option<usize>,

	#[clap(
		long = "relay-set",
		name = "RELAY_SET",
		about = "Set of transactions to relay. SET may be: cheap - Relay any transaction in the queue (this may include invalid transactions); strict - Relay only executed transactions (this guarantees we don't relay invalid transactions, but means we relay nothing if not mining); lenient - Same as strict when mining, and cheap when not.",
		help_heading = Some(SEALING_MINING),
	)]
	pub relay_set: Option<String>,

	#[clap(
		long = "usd-per-tx",
		name = "USD_PER_TX",
		about = "Amount of USD to be paid for a basic transaction. The minimum gas price is set accordingly.",
		help_heading = Some(SEALING_MINING),
	)]
	pub usd_per_tx: Option<String>,

	#[clap(
		long = "usd-per-eth",
		name = "USD_PER_ETH_SOURCE",
		about = "USD value of a single ETH. SOURCE may be either an amount in USD, a web service or 'auto' to use each web service in turn and fallback on the last known good value.",
		help_heading = Some(SEALING_MINING),
	)]
	pub usd_per_eth: Option<String>,

	#[clap(
		long = "price-update-period",
		name = "PRICE_UPDATE_T",
		about = "PRICE_UPDATE_T will be allowed to pass between each gas price update. PRICE_UPDATE_T may be daily, hourly, a number of seconds, or a time string of the form \"2 days\", \"30 minutes\" etc..",
		help_heading = Some(SEALING_MINING),
	)]
	pub price_update_period: Option<String>,

	#[clap(
		long = "gas-floor-target",
		name = "GAS_FLOOR",
		about = "Amount of gas per block to target when sealing a new block.",
		help_heading = Some(SEALING_MINING),
	)]
	pub gas_floor_target: Option<String>,

	#[clap(
		long = "gas-cap",
		name = "GAS_CAP",
		about = "A cap on how large we will raise the gas limit per block due to transaction volume.",
		help_heading = Some(SEALING_MINING),
	)]
	pub gas_cap: Option<String>,

	#[clap(
		long = "tx-queue-mem-limit",
		name = "TX_QUEUE_LIMIT_MB",
		about = "Maximum amount of memory that can be used by the transaction queue. Setting this parameter to 0 disables limiting.",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_mem_limit: Option<u32>,

	#[clap(
		long = "tx-queue-size",
		name = "TX_QUEUE_SIZE_LIMIT",
		about = "Maximum amount of transactions in the queue (waiting to be included in next block).",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_size: Option<usize>,

	#[clap(
		long = "tx-queue-per-sender",
		name = "TX_QUEUE_PER_SENDER_LIMIT",
		about = "Maximum number of transactions per sender in the queue. By default it's 1% of the entire queue, but not less than 16.",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_per_sender: Option<usize>,

	#[clap(
		long = "tx-queue-locals",
		name = "TX_QUEUE_LOCAL_ACCOUNTS",
		about = "Specify local accounts for which transactions are prioritized in the queue. ACCOUNTS is a comma-delimited list of addresses.",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_locals: Option<String>,

	#[clap(
		long = "tx-queue-strategy",
		name = "TX_QUEUE_S",
		about = "Prioritization strategy used to order transactions in the queue. S may be: gas_price - Prioritize txs with high gas price",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_queue_strategy: Option<String>,

	#[clap(
		long = "stratum-interface",
		name = "STRATUM_IP",
		about = "Interface address for Stratum server.",
		help_heading = Some(SEALING_MINING),
	)]
	pub stratum_interface: Option<String>,

	#[clap(
		long = "stratum-port",
		name = "STRATUM_PORT",
		about = "Port for Stratum server to listen on.",
		help_heading = Some(SEALING_MINING),
	)]
	pub stratum_port: Option<u16>,

	#[clap(
		long = "min-gas-price",
		name = "MIN_GAS_PRICE_STRING",
		about = "Minimum amount of Wei per GAS to be paid for a transaction to be accepted for mining. Overrides --usd-per-tx.",
		help_heading = Some(SEALING_MINING),
	)]
	pub min_gas_price: Option<u64>,

	#[clap(
		long = "gas-price-percentile",
		name = "PCT",
		about = "Set PCT percentile gas price value from last 100 blocks as default gas price when sending transactions.",
		help_heading = Some(SEALING_MINING),
	)]
	pub gas_price_percentile: Option<usize>,

	#[clap(
		long,
		name = "ADDRESS",
		about = "Specify the block author (aka \"coinbase\") address for sending block rewards from sealed blocks. NOTE: MINING WILL NOT WORK WITHOUT THIS OPTION.",
		help_heading = Some(SEALING_MINING),
	)]
	pub author: Option<String>, // Sealing / Mining Option

	#[clap(
		long = "engine-signer",
		name = "ENGINE_SIGNER_ADDRESS",
		about = "Specify the address which should be used to sign consensus messages and issue blocks. Relevant only to non-PoW chains.",
		help_heading = Some(SEALING_MINING),
	)]
	pub engine_signer: Option<String>,

	#[clap(
		long = "tx-gas-limit",
		name = "TX_GAS_LIMIT",
		about = "Apply a limit of GAS as the maximum amount of gas a single transaction may have for it to be mined.",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_gas_limit: Option<String>,

	#[clap(
		long = "tx-time-limit",
		name = "TX_TIME_LIMIT_MS",
		about = "Maximal time for processing single transaction. If enabled senders of transactions offending the limit will get other transactions penalized.",
		help_heading = Some(SEALING_MINING),
	)]
	pub tx_time_limit: Option<u64>,

	#[clap(
		long = "extra-data",
		name = "EXTRA_DATA_STRING",
		about = "Specify a custom extra-data for authored blocks, no more than 32 characters.",
		help_heading = Some(SEALING_MINING),
	)]
	pub extra_data: Option<String>,

	#[clap(
		long = "notify-work",
		name = "NOTIFY_WORK_URLS",
		about = "URLs to which work package notifications are pushed. URLS should be a comma-delimited list of HTTP URLs.",
		help_heading = Some(SEALING_MINING),
	)]
	pub notify_work: Option<String>,

	#[clap(
		long = "stratum-secret",
		name = "STARTUM_SECRET_STRING",
		about = "Secret for authorizing Stratum server for peers.",
		help_heading = Some(SEALING_MINING),
	)]
	pub stratum_secret: Option<String>,

	#[clap(
		long = "max-round-blocks-to-import",
		name = "MAX_ROUND_BLOCKS_S",
		about = "Maximal number of blocks to import for each import round.",
		help_heading = Some(SEALING_MINING),
	)]
	pub max_round_blocks_to_import: Option<usize>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct InternalOptions {
	#[clap(
		long = "can-restart",
		about = "Executable will auto-restart if exiting with 69",
		help_heading = Some(INTERNAL),
	)]
	pub can_restart: bool,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct MiscellaneousOptions {
	#[clap(
		long = "no-color",
		about = "Don't use terminal color codes in output.",
		help_heading = Some(MISCELLANEOUS),
	)]
	pub no_color: bool,

	// version flag is automatically provided by structopt
	#[clap(
		long = "no-config",
		about = "Don't load a configuration file.",
		help_heading = Some(MISCELLANEOUS),
	)]
	pub no_config: bool,

	#[clap(
		short = "l",
		long,
		name = "LOGGING",
		about = "Specify the general logging level (error, warn, info, debug or trace). It can also be set for a specific module, example: '-l sync=debug,rpc=trace'",
		help_heading = Some(MISCELLANEOUS),
	)]
	pub logging: Option<String>,

	#[clap(
		long = "log-file",
		name = "LOG_FILENAME",
		about = "Specify a filename into which logging should be appended",
		help_heading = Some(MISCELLANEOUS),
	)]
	pub log_file: Option<String>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct FootPrintOptions {
	#[clap(
		long = "scale-verifiers",
		about = "Automatically scale amount of verifier threads based on workload. Not guaranteed to be faster.",
		help_heading = Some(FOOTPRINT),
	)]
	pub scale_verifiers: bool,

	#[clap(
		long,
		name = "TRACING_BOOL",
		about = "Indicates if full transaction tracing should be enabled. Works only if client had been fully synced with tracing enabled. BOOL may be one of auto, on, off. auto uses last used value of this option (off if it does not exist).",
		help_heading = Some(FOOTPRINT),
	)]
	pub tracing: Option<String>,

	#[clap(
		long,
		name = "PRUNING_METHOD",
		about = "Configure pruning of the state/storage trie. PRUNING_METHOD may be one of auto, archive, fast: archive - keep all state trie data. No pruning. fast - maintain journal overlay. Fast but 50MB used. auto - use the method most recently synced or default to fast if none synced.",
		help_heading = Some(FOOTPRINT),
	)]
	pub pruning: Option<String>,

	#[clap(
		long = "pruning-history",
		name = "PRUNING_HISTORY_NUM",
		about = "Set a minimum number of recent states to keep in memory when pruning is active.",
		help_heading = Some(FOOTPRINT),
	)]
	pub pruning_history: Option<u64>,

	#[clap(
		long = "pruning-memory",
		name = "PRUNING_MEMORY_MB",
		about = "The ideal amount of memory in megabytes to use to store recent states. As many states as possible will be kept within this limit, and at least --pruning-history states will always be kept.",
		help_heading = Some(FOOTPRINT),
	)]
	pub pruning_memory: Option<usize>,

	#[clap(
		long = "cache-size-db",
		name = "CACHE_SIZE_DB_MB",
		about = "Override database cache size.",
		help_heading = Some(FOOTPRINT),
	)]
	pub cache_size_db: Option<u32>,

	#[clap(
		long = "cache-size-blocks",
		name = "CACHE_SIZE_BLOCKS_MB",
		about = "Specify the preferred size of the blockchain cache in megabytes.",
		help_heading = Some(FOOTPRINT),
	)]
	pub cache_size_blocks: Option<u32>,

	#[clap(
		long = "cache-size-queue",
		name = "CACHE_SIZE_QUEUE_MB",
		about = "Specify the maximum size of memory to use for block queue.",
		help_heading = Some(FOOTPRINT),
	)]
	pub cache_size_queue: Option<u32>,

	#[clap(
		long = "cache-size-state",
		name = "CACHE_SIZE_STATE",
		about = "Specify the maximum size of memory to use for the state cache.",
		help_heading = Some(FOOTPRINT),
	)]
	pub cache_size_state: Option<u32>,

	#[clap(
		long = "db-compaction",
		name = "DB_COMPACTION_TYPE",
		about = "Database compaction type. TYPE may be one of: ssd - suitable for SSDs and fast HDDs; hdd - suitable for slow HDDs; auto - determine automatically.",
		help_heading = Some(FOOTPRINT),
	)]
	pub db_compaction: Option<String>,

	#[clap(
		long = "fat-db",
		name = "FAT_DB_BOOL",
		about = "Build appropriate information to allow enumeration of all accounts and storage keys. Doubles the size of the state database. BOOL may be one of on, off or auto.",
		help_heading = Some(FOOTPRINT),
	)]
	pub fat_db: Option<String>,

	#[clap(
		long = "cache-size",
		name = "CACHE_SIZE_MB",
		about = "Set total amount of discretionary memory to use for the entire system, overrides other cache and queue options.",
		help_heading = Some(FOOTPRINT),
	)]
	pub cache_size: Option<u32>,

	#[clap(
		name = "NUM_VERIFIERS_INT",
		long = "num-verifiers",
		about = "Amount of verifier threads to use or to begin with, if verifier auto-scaling is enabled.",
		help_heading = Some(FOOTPRINT),
	)]
	pub num_verifiers: Option<usize>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ImportExportOptions {
	#[clap(
		long = "no-seal-check",
		about = "Skip block seal check.",
		help_heading = Some(IMPORT_EXPORT),
	)]
	pub no_seal_check: bool,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SnapshotOptions {
	#[clap(
		long = "no-periodic-snapshots",
		about = "Disable automated snapshots which usually occur once every 5000 blocks.",
		help_heading = Some(SNAPSHOT),
	)]
	pub no_periodic_snapshot: bool,

	#[clap(
		long = "snapshot-threads",
		name = "SNAPSHOT_THREADS_NUM",
		about = "Enables multiple threads for snapshots creation.",
		help_heading = Some(SNAPSHOT),
	)]
	pub snapshot_threads: Option<usize>,
}

#[derive(Clap, Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct LegacyOptions {
	// TODO: These options were hidden from config, so should we not include them?
	#[clap(
		long,
		about = "Run in Geth-compatibility mode. Sets the IPC path to be the same as Geth's. Overrides the --ipc-path and --ipcpath options. Alters RPCs to reflect Geth bugs. Includes the personal_ RPC by default.",
		help_heading = Some(LEGACY),
	)]
	pub geth: bool,

	#[clap(
		long = "import-geth-keys",
		about = "Attempt to import keys from Geth client.",
		help_heading = Some(LEGACY),
	)]
	pub import_geth_keys: bool,
}
