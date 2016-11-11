// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use util::version;
use docopt::Docopt;

pub const USAGE: &'static str = r#"
Parity. Ethereum Client.
  By Wood/Paronyan/Kotewicz/Drwięga/Volf.
  Copyright 2015, 2016 Ethcore (UK) Limited

Usage:
  parity [options]
  parity ui [options]
  parity daemon <pid-file> [options]
  parity account (new | list ) [options]
  parity account import <path>... [options]
  parity wallet import <path> --password FILE [options]
  parity import [ <file> ] [options]
  parity export [ <file> ] [options]
  parity signer new-token [options]
  parity snapshot <file> [options]
  parity restore <file> [options]

Operating Options:
  --mode MODE              Set the operating mode. MODE can be one of:
                           active - Parity continuously syncs the chain.
                           passive - Parity syncs initially, then sleeps and
                           wakes regularly to resync.
                           dark - Parity syncs only when an external interface
                           is active. [default: active].
  --mode-timeout SECS      Specify the number of seconds before inactivity
                           timeout occurs when mode is dark or passive
                           [default: 300].
  --mode-alarm SECS        Specify the number of seconds before auto sleep
                           reawake timeout occurs when mode is passive
                           [default: 3600].
  --chain CHAIN            Specify the blockchain type. CHAIN may be either a
                           JSON chain specification file or olympic, frontier,
                           homestead, mainnet, morden, classic or testnet
                           [default: homestead].
  -d --db-path PATH        Specify the database & configuration directory path
                           [default: $HOME/.parity].
  --keys-path PATH         Specify the path for JSON key files to be found
                           [default: $HOME/.parity/keys].
  --identity NAME          Specify your node's name.

Account Options:
  --unlock ACCOUNTS        Unlock ACCOUNTS for the duration of the execution.
                           ACCOUNTS is a comma-delimited list of addresses.
                           Implies --no-signer.
  --password FILE          Provide a file containing a password for unlocking
                           an account.
  --keys-iterations NUM    Specify the number of iterations to use when
                           deriving key from the password (bigger is more
                           secure) [default: 10240].
  --no-import-keys         Do not import keys from legacy clients.
  --force-signer           Enable Trusted Signer WebSocket endpoint used by
                           Signer UIs, even when --unlock is in use.
  --no-signer              Disable Trusted Signer WebSocket endpoint used by
                           Signer UIs.
  --signer-port PORT       Specify the port of Trusted Signer server
                           [default: 8180].
  --signer-path PATH       Specify directory where Signer UIs tokens should
                           be stored. [default: $HOME/.parity/signer]
  --signer-no-validation   Disable Origin and Host headers validation for
                           Trusted Signer. WARNING: INSECURE. Used only for
                           development.

Networking Options:
  --no-network             Disable p2p networking.
  --port PORT              Override the port on which the node should listen
                           [default: 30303].
  --min-peers NUM          Try to maintain at least NUM peers [default: 25].
  --max-peers NUM          Allow up to that many peers [default: 50].
  --nat METHOD             Specify method to use for determining public
                           address. Must be one of: any, none, upnp,
                           extip:<IP> [default: any].
  --network-id INDEX       Override the network identifier from the chain we
                           are on.
  --bootnodes NODES        Override the bootnodes from our chain. NODES should
                           be comma-delimited enodes.
  --no-discovery           Disable new peer discovery.
  --node-key KEY           Specify node secret key, either as 64-character hex
                           string or input to SHA3 operation.
  --reserved-peers FILE    Provide a file containing enodes, one per line.
                           These nodes will always have a reserved slot on top
                           of the normal maximum peers.
  --reserved-only          Connect only to reserved nodes.

API and Console Options:
  --no-jsonrpc             Disable the JSON-RPC API server.
  --jsonrpc-port PORT      Specify the port portion of the JSONRPC API server
                           [default: 8545].
  --jsonrpc-interface IP   Specify the hostname portion of the JSONRPC API
                           server, IP should be an interface's IP address, or
                           all (all interfaces) or local [default: local].
  --jsonrpc-cors URL       Specify CORS header for JSON-RPC API responses.
  --jsonrpc-apis APIS      Specify the APIs available through the JSONRPC
                           interface. APIS is a comma-delimited list of API
                           name. Possible name are web3, eth, net, personal,
                           ethcore, ethcore_set, traces, rpc.
                           [default: web3,eth,net,ethcore,personal,traces,rpc].
  --jsonrpc-hosts HOSTS    List of allowed Host header values. This option will
                           validate the Host header sent by the browser, it
                           is additional security against some attack
                           vectors. Special options: "all", "none",
                           [default: none].

  --no-ipc                 Disable JSON-RPC over IPC service.
  --ipc-path PATH          Specify custom path for JSON-RPC over IPC service
                           [default: $HOME/.parity/jsonrpc.ipc].
  --ipc-apis APIS          Specify custom API set available via JSON-RPC over
                           IPC [default: web3,eth,net,ethcore,personal,traces,rpc].

  --dapps-port PORT        Specify the port portion of the Dapps server
                           [default: 8080].
  --dapps-interface IP     Specify the hostname portion of the Dapps
                           server, IP should be an interface's IP address,
                           or local [default: local].
  --dapps-hosts HOSTS      List of allowed Host header values. This option will
                           validate the Host header sent by the browser, it
                           is additional security against some attack
                           vectors. Special options: "all", "none",
                           [default: none].
  --dapps-user USERNAME    Specify username for Dapps server. It will be
                           used in HTTP Basic Authentication Scheme.
                           If --dapps-pass is not specified you will be
                           asked for password on startup.
  --dapps-pass PASSWORD    Specify password for Dapps server. Use only in
                           conjunction with --dapps-user.
  --dapps-path PATH        Specify directory where dapps should be installed.
                           [default: $HOME/.parity/dapps]

Sealing/Mining Options:
  --author ADDRESS         Specify the block author (aka "coinbase") address
                           for sending block rewards from sealed blocks.
                           NOTE: MINING WILL NOT WORK WITHOUT THIS OPTION.
  --force-sealing          Force the node to author new blocks as if it were
                           always sealing/mining.
  --reseal-on-txs SET      Specify which transactions should force the node
                           to reseal a block. SET is one of:
                           none - never reseal on new transactions;
                           own - reseal only on a new local transaction;
                           ext - reseal only on a new external transaction;
                           all - reseal on all new transactions [default: own].
  --reseal-min-period MS   Specify the minimum time between reseals from
                           incoming transactions. MS is time measured in
                           milliseconds [default: 2000].
  --work-queue-size ITEMS  Specify the number of historical work packages
                           which are kept cached lest a solution is found for
                           them later. High values take more memory but result
                           in fewer unusable solutions [default: 5].
  --tx-gas-limit GAS       Apply a limit of GAS as the maximum amount of gas
                           a single transaction may have for it to be mined.
  --relay-set SET          Set of transactions to relay. SET may be:
                           cheap - Relay any transaction in the queue (this
                           may include invalid transactions);
                           strict - Relay only executed transactions (this
                           guarantees we don't relay invalid transactions, but
                           means we relay nothing if not mining);
                           lenient - Same as strict when mining, and cheap
                           when not [default: cheap].
  --usd-per-tx USD         Amount of USD to be paid for a basic transaction
                           [default: 0.005]. The minimum gas price is set
                           accordingly.
  --usd-per-eth SOURCE     USD value of a single ETH. SOURCE may be either an
                           amount in USD, a web service or 'auto' to use each
                           web service in turn and fallback on the last known
                           good value [default: auto].
  --price-update-period T  T will be allowed to pass between each gas price
                           update. T may be daily, hourly, a number of seconds,
                           or a time string of the form "2 days", "30 minutes"
                           etc. [default: hourly].
  --gas-floor-target GAS   Amount of gas per block to target when sealing a new
                           block [default: 4700000].
  --gas-cap GAS            A cap on how large we will raise the gas limit per
                           block due to transaction volume [default: 6283184].
  --extra-data STRING      Specify a custom extra-data for authored blocks, no
                           more than 32 characters.
  --tx-queue-size LIMIT    Maximum amount of transactions in the queue (waiting
                           to be included in next block) [default: 2048].
  --tx-queue-strategy S    Prioritization strategy used to order transactions
                           in the queue. S may be:
                           gas - Prioritize txs with low gas limit;
                           gas_price - Prioritize txs with high gas price;
                           gas_factor - Prioritize txs using gas price
                           and gas limit ratio [default: gas_factor].
  --tx-queue-gas LIMIT     Maximum amount of total gas for external transactions in
                           the queue. LIMIT can be either an amount of gas or
                           'auto' or 'off'. 'auto' sets the limit to be 20x
                           the current block gas limit. [default: auto].
  --remove-solved          Move solved blocks from the work package queue
                           instead of cloning them. This gives a slightly
                           faster import speed, but means that extra solutions
                           submitted for the same work package will go unused.
  --notify-work URLS       URLs to which work package notifications are pushed.
                           URLS should be a comma-delimited list of HTTP URLs.

Footprint Options:
  --tracing BOOL           Indicates if full transaction tracing should be
                           enabled. Works only if client had been fully synced
                           with tracing enabled. BOOL may be one of auto, on,
                           off. auto uses last used value of this option (off
                           if it does not exist) [default: auto].
  --pruning METHOD         Configure pruning of the state/storage trie. METHOD
                           may be one of auto, archive, fast:
                           archive - keep all state trie data. No pruning.
                           fast - maintain journal overlay. Fast but 50MB used.
                           auto - use the method most recently synced or
                           default to fast if none synced [default: auto].
  --pruning-history NUM    Set a number of recent states to keep when pruning
                           is active. [default: 64].
  --cache-size-db MB       Override database cache size [default: 64].
  --cache-size-blocks MB   Specify the prefered size of the blockchain cache in
                           megabytes [default: 8].
  --cache-size-queue MB    Specify the maximum size of memory to use for block
                           queue [default: 50].
  --cache-size MB          Set total amount of discretionary memory to use for
                           the entire system, overrides other cache and queue
                           options.
  --fast-and-loose         Disables DB WAL, which gives a significant speed up
                           but means an unclean exit is unrecoverable.
  --db-compaction TYPE     Database compaction type. TYPE may be one of:
                           ssd - suitable for SSDs and fast HDDs;
                           hdd - suitable for slow HDDs [default: ssd].
  --fat-db                 Fat database.

Import/Export Options:
  --from BLOCK             Export from block BLOCK, which may be an index or
                           hash [default: 1].
  --to BLOCK               Export to (including) block BLOCK, which may be an
                           index, hash or 'latest' [default: latest].
  --format FORMAT          For import/export in given format. FORMAT must be
                           one of 'hex' and 'binary'.

Snapshot Options:
  --at BLOCK               Take a snapshot at the given block, which may be an
                           index, hash, or 'latest'. Note that taking snapshots at
                           non-recent blocks will only work with --pruning archive
                           [default: latest]

Virtual Machine Options:
  --jitvm                  Enable the JIT VM.

Legacy Options:
  --geth                   Run in Geth-compatibility mode. Sets the IPC path
                           to be the same as Geth's. Overrides the --ipc-path
                           and --ipcpath options. Alters RPCs to reflect Geth
                           bugs.
  --testnet                Geth-compatible testnet mode. Equivalent to --chain
                           testnet --keys-path $HOME/parity/testnet-keys.
                           Overrides the --keys-path option.
  --datadir PATH           Equivalent to --db-path PATH.
  --networkid INDEX        Equivalent to --network-id INDEX.
  --peers NUM              Equivalent to --min-peers NUM.
  --nodekey KEY            Equivalent to --node-key KEY.
  --nodiscover             Equivalent to --no-discovery.
  -j --jsonrpc             Does nothing; JSON-RPC is on by default now.
  --jsonrpc-off            Equivalent to --no-jsonrpc.
  -w --webapp              Does nothing; dapps server is on by default now.
  --dapps-off              Equivalent to --no-dapps.
  --rpc                    Does nothing; JSON-RPC is on by default now.
  --rpcaddr IP             Equivalent to --jsonrpc-interface IP.
  --rpcport PORT           Equivalent to --jsonrpc-port PORT.
  --rpcapi APIS            Equivalent to --jsonrpc-apis APIS.
  --rpccorsdomain URL      Equivalent to --jsonrpc-cors URL.
  --ipcdisable             Equivalent to --no-ipc.
  --ipc-off                Equivalent to --no-ipc.
  --ipcapi APIS            Equivalent to --ipc-apis APIS.
  --ipcpath PATH           Equivalent to --ipc-path PATH.
  --gasprice WEI           Minimum amount of Wei per GAS to be paid for a
                           transaction to be accepted for mining. Overrides
                           --basic-tx-usd.
  --etherbase ADDRESS      Equivalent to --author ADDRESS.
  --extradata STRING       Equivalent to --extra-data STRING.
  --cache MB               Equivalent to --cache-size MB.
  --no-dapps               Disable the Dapps server (e.g. status page).

Miscellaneous Options:
  -l --logging LOGGING     Specify the logging level. Must conform to the same
                           format as RUST_LOG.
  --log-file FILENAME      Specify a filename into which logging should be
                           directed.
  --no-color               Don't use terminal color codes in output.
  -v --version             Show information about version.
  -h --help                Show this screen.
"#;

#[derive(Debug, PartialEq, RustcDecodable)]
pub struct Args {
	pub cmd_daemon: bool,
	pub cmd_account: bool,
	pub cmd_wallet: bool,
	pub cmd_new: bool,
	pub cmd_list: bool,
	pub cmd_export: bool,
	pub cmd_import: bool,
	pub cmd_signer: bool,
	pub cmd_new_token: bool,
	pub cmd_snapshot: bool,
	pub cmd_restore: bool,
	pub cmd_ui: bool,
	pub arg_pid_file: String,
	pub arg_file: Option<String>,
	pub arg_path: Vec<String>,
	pub flag_mode: String,
	pub flag_mode_timeout: u64,
	pub flag_mode_alarm: u64,
	pub flag_chain: String,
	pub flag_db_path: String,
	pub flag_identity: String,
	pub flag_unlock: Option<String>,
	pub flag_password: Vec<String>,
	pub flag_keys_path: String,
	pub flag_keys_iterations: u32,
	pub flag_no_import_keys: bool,
	pub flag_bootnodes: Option<String>,
	pub flag_network_id: Option<usize>,
	pub flag_pruning: String,
	pub flag_pruning_history: u64,
	pub flag_tracing: String,
	pub flag_port: u16,
	pub flag_min_peers: u16,
	pub flag_max_peers: u16,
	pub flag_no_discovery: bool,
	pub flag_nat: String,
	pub flag_node_key: Option<String>,
	pub flag_reserved_peers: Option<String>,
	pub flag_reserved_only: bool,

	pub flag_cache_size_db: u32,
	pub flag_cache_size_blocks: u32,
	pub flag_cache_size_queue: u32,
	pub flag_cache_size: Option<u32>,
	pub flag_cache: Option<u32>,
	pub flag_fast_and_loose: bool,

	pub flag_no_jsonrpc: bool,
	pub flag_jsonrpc_interface: String,
	pub flag_jsonrpc_port: u16,
	pub flag_jsonrpc_cors: Option<String>,
	pub flag_jsonrpc_hosts: String,
	pub flag_jsonrpc_apis: String,
	pub flag_no_ipc: bool,
	pub flag_ipc_path: String,
	pub flag_ipc_apis: String,
	pub flag_no_dapps: bool,
	pub flag_dapps_port: u16,
	pub flag_dapps_interface: String,
	pub flag_dapps_hosts: String,
	pub flag_dapps_user: Option<String>,
	pub flag_dapps_pass: Option<String>,
	pub flag_dapps_path: String,
	pub flag_force_signer: bool,
	pub flag_no_signer: bool,
	pub flag_signer_port: u16,
	pub flag_signer_path: String,
	pub flag_signer_no_validation: bool,
	pub flag_force_sealing: bool,
	pub flag_reseal_on_txs: String,
	pub flag_reseal_min_period: u64,
	pub flag_work_queue_size: usize,
	pub flag_remove_solved: bool,
	pub flag_tx_gas_limit: Option<String>,
	pub flag_relay_set: String,
	pub flag_author: Option<String>,
	pub flag_usd_per_tx: String,
	pub flag_usd_per_eth: String,
	pub flag_price_update_period: String,
	pub flag_gas_floor_target: String,
	pub flag_gas_cap: String,
	pub flag_extra_data: Option<String>,
	pub flag_tx_queue_size: usize,
	pub flag_tx_queue_strategy: String,
	pub flag_tx_queue_gas: String,
	pub flag_notify_work: Option<String>,
	pub flag_logging: Option<String>,
	pub flag_version: bool,
	pub flag_from: String,
	pub flag_to: String,
	pub flag_at: String,
	pub flag_format: Option<String>,
	pub flag_jitvm: bool,
	pub flag_log_file: Option<String>,
	pub flag_no_color: bool,
	pub flag_no_network: bool,
	// legacy...
	pub flag_geth: bool,
	pub flag_nodekey: Option<String>,
	pub flag_nodiscover: bool,
	pub flag_peers: Option<u16>,
	pub flag_datadir: Option<String>,
	pub flag_extradata: Option<String>,
	pub flag_etherbase: Option<String>,
	pub flag_gasprice: Option<String>,
	pub flag_jsonrpc: bool,
	pub flag_webapp: bool,
	pub flag_rpc: bool,
	pub flag_rpcaddr: Option<String>,
	pub flag_rpcport: Option<u16>,
	pub flag_rpccorsdomain: Option<String>,
	pub flag_rpcapi: Option<String>,
	pub flag_testnet: bool,
	pub flag_networkid: Option<usize>,
	pub flag_ipcdisable: bool,
	pub flag_ipc_off: bool,
	pub flag_jsonrpc_off: bool,
	pub flag_dapps_off: bool,
	pub flag_ipcpath: Option<String>,
	pub flag_ipcapi: Option<String>,
	pub flag_db_compaction: String,
	pub flag_fat_db: bool,
}

impl Default for Args {
	fn default() -> Self {
		Docopt::new(USAGE).unwrap().argv(&[] as &[&str]).decode().unwrap()
	}
}

pub fn print_version() -> String {
	format!("\
Parity
  version {}
Copyright 2015, 2016 Ethcore (UK) Limited
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

By Wood/Paronyan/Kotewicz/Drwięga/Volf.\
", version())
}

