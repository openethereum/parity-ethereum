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

pub const USAGE: &'static str = r#"
Parity. Ethereum Client.
  By Wood/Paronyan/Kotewicz/Drwięga/Volf.
  Copyright 2015, 2016 Ethcore (UK) Limited

Usage:
  parity daemon <pid-file> [options]
  parity account (new | list) [options]
  parity import [ <file> ] [options]
  parity export [ <file> ] [options]
  parity [options]

Protocol Options:
  --chain CHAIN            Specify the blockchain type. CHAIN may be either a
                           JSON chain specification file or olympic, frontier,
                           homestead, mainnet, morden, or testnet
                           [default: homestead].
  -d --db-path PATH        Specify the database & configuration directory path
                           [default: $HOME/.parity].
  --keys-path PATH         Specify the path for JSON key files to be found
                           [default: $HOME/.parity/keys].
  --identity NAME          Specify your node's name.

Account Options:
  --unlock ACCOUNTS        Unlock ACCOUNTS for the duration of the execution.
                           ACCOUNTS is a comma-delimited list of addresses.
  --password FILE          Provide a file containing a password for unlocking
                           an account.
  --keys-iterations NUM    Specify the number of iterations to use when
                           deriving key from the password (bigger is more
                           secure) [default: 10240].
  --no-import-keys         Do not import keys from legacy clients.

Networking Options:
  --port PORT              Override the port on which the node should listen
                           [default: 30303].
  --peers NUM              Try to maintain that many peers [default: 25].
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

API and Console Options:
  --jsonrpc-off            Disable the JSON-RPC API server.
  --jsonrpc-port PORT      Specify the port portion of the JSONRPC API server
                           [default: 8545].
  --jsonrpc-interface IP   Specify the hostname portion of the JSONRPC API
                           server, IP should be an interface's IP address, or
                           all (all interfaces) or local [default: local].
  --jsonrpc-cors URL       Specify CORS header for JSON-RPC API responses.
  --jsonrpc-apis APIS      Specify the APIs available through the JSONRPC
                           interface. APIS is a comma-delimited list of API
                           name. Possible name are web3, eth, net, personal,
                           ethcore, traces.
                           [default: web3,eth,net,personal,ethcore,traces].

  --ipc-off                Disable JSON-RPC over IPC service.
  --ipc-path PATH          Specify custom path for JSON-RPC over IPC service
                           [default: $HOME/.parity/jsonrpc.ipc].
  --ipc-apis APIS          Specify custom API set available via JSON-RPC over
                           IPC [default: web3,eth,net,personal,ethcore].

  --dapps-off              Disable the Dapps server (e.g. status page).
  --dapps-port PORT        Specify the port portion of the Dapps server
                           [default: 8080].
  --dapps-interface IP     Specify the hostname portion of the Dapps
                           server, IP should be an interface's IP address, or
                           all (all interfaces) or local [default: local].
  --dapps-user USERNAME    Specify username for Dapps server. It will be
                           used in HTTP Basic Authentication Scheme.
                           If --dapps-pass is not specified you will be
                           asked for password on startup.
  --dapps-pass PASSWORD    Specify password for Dapps server. Use only in
                           conjunction with --dapps-user.

Sealing/Mining Options:
  --force-sealing          Force the node to author new blocks as if it were
                           always sealing/mining.
  --usd-per-tx USD         Amount of USD to be paid for a basic transaction
                           [default: 0.005]. The minimum gas price is set
                           accordingly.
  --usd-per-eth SOURCE     USD value of a single ETH. SOURCE may be either an
                           amount in USD, a web service or 'auto' to use each
                           web service in turn and fallback on the last known
                           good value [default: auto].
  --gas-floor-target GAS   Amount of gas per block to target when sealing a new
                           block [default: 4712388].
  --author ADDRESS         Specify the block author (aka "coinbase") address
                           for sending block rewards from sealed blocks
                           [default: 0037a6b811ffeb6e072da21179d11b1406371c63].
  --extra-data STRING      Specify a custom extra-data for authored blocks, no
                           more than 32 characters.
  --tx-limit LIMIT         Limit of transactions kept in the queue (waiting to
                           be included in next block) [default: 1024].

Footprint Options:
  --tracing BOOL           Indicates if full transaction tracing should be
                           enabled. Works only if client had been fully synced
                           with tracing enabled. BOOL may be one of auto, on,
                           off. auto uses last used value of this option (off
                           if it does not exist) [default: auto].
  --pruning METHOD         Configure pruning of the state/storage trie. METHOD
                           may be one of auto, archive, fast, basic, light:
                           archive - keep all state trie data. No pruning.
                           fast - maintain journal overlay. Fast but 50MB used.
                           basic - reference count in disk DB. Slow, light, and
                           experimental!
                           light - early merges with partial tracking. Fast,
                           light, and experimental!
                           auto - use the method most recently synced or
                           default to archive if none synced [default: auto].
  --cache-pref-size BYTES  Specify the prefered size of the blockchain cache in
                           bytes [default: 16384].
  --cache-max-size BYTES   Specify the maximum size of the blockchain cache in
                           bytes [default: 262144].
  --queue-max-size BYTES   Specify the maximum size of memory to use for block
                           queue [default: 52428800].
  --cache MEGABYTES        Set total amount of discretionary memory to use for
                           the entire system, overrides other cache and queue
                           options.

Import/Export Options:
  --from BLOCK             Export from block BLOCK, which may be an index or
                           hash [default: 1].
  --to BLOCK               Export to (including) block BLOCK, which may be an
                           index, hash or 'latest' [default: latest].
  --format FORMAT          For import/export in given format. FORMAT must be
                           one of 'hex' and 'binary'.

Virtual Machine Options:
  --jitvm                  Enable the JIT VM.

Legacy Options:
  --geth                   Run in Geth-compatibility mode. Currently just sets
                           the IPC path to be the same as Geth's. Overrides
                           the --ipc-path/--ipcpath options.
  --testnet                Geth-compatible testnet mode. Equivalent to --chain
                           testnet --keys-path $HOME/parity/testnet-keys.
                           Overrides the --keys-path option.
  --datadir PATH           Equivalent to --db-path PATH.
  --networkid INDEX        Equivalent to --network-id INDEX.
  --maxpeers COUNT         Equivalent to --peers COUNT.
  --nodekey KEY            Equivalent to --node-key KEY.
  --nodiscover             Equivalent to --no-discovery.
  -j --jsonrpc             Does nothing; JSON-RPC is on by default now.
  -w --webapp              Does nothing; dapps server is on by default now.
  --rpc                    Does nothing; JSON-RPC is on by default now.
  --rpcaddr IP             Equivalent to --jsonrpc-interface IP.
  --rpcport PORT           Equivalent to --jsonrpc-port PORT.
  --rpcapi APIS            Equivalent to --jsonrpc-apis APIS.
  --rpccorsdomain URL      Equivalent to --jsonrpc-cors URL.
  --ipcdisable             Equivalent to --ipc-off.
  --ipcapi APIS            Equivalent to --ipc-apis APIS.
  --ipcpath PATH           Equivalent to --ipc-path PATH.
  --gasprice WEI           Minimum amount of Wei per GAS to be paid for a
                           transaction to be accepted for mining. Overrides
                           --basic-tx-usd.
  --etherbase ADDRESS      Equivalent to --author ADDRESS.
  --extradata STRING       Equivalent to --extra-data STRING.

Miscellaneous Options:
  -l --logging LOGGING     Specify the logging level. Must conform to the same
                           format as RUST_LOG.
  --no-color               Don't use terminal color codes in output.
  -v --version             Show information about version.
  -h --help                Show this screen.
"#;

#[derive(Debug, RustcDecodable)]
pub struct Args {
	pub cmd_daemon: bool,
	pub cmd_account: bool,
	pub cmd_new: bool,
	pub cmd_list: bool,
	pub cmd_export: bool,
	pub cmd_import: bool,
	pub arg_pid_file: String,
	pub arg_file: Option<String>,
	pub flag_chain: String,
	pub flag_db_path: String,
	pub flag_identity: String,
	pub flag_unlock: Option<String>,
	pub flag_password: Vec<String>,
	pub flag_cache: Option<usize>,
	pub flag_keys_path: String,
	pub flag_keys_iterations: u32,
	pub flag_no_import_keys: bool,
	pub flag_bootnodes: Option<String>,
	pub flag_network_id: Option<String>,
	pub flag_pruning: String,
	pub flag_tracing: String,
	pub flag_port: u16,
	pub flag_peers: usize,
	pub flag_no_discovery: bool,
	pub flag_nat: String,
	pub flag_node_key: Option<String>,
	pub flag_cache_pref_size: usize,
	pub flag_cache_max_size: usize,
	pub flag_queue_max_size: usize,
	pub flag_jsonrpc_off: bool,
	pub flag_jsonrpc_interface: String,
	pub flag_jsonrpc_port: u16,
	pub flag_jsonrpc_cors: Option<String>,
	pub flag_jsonrpc_apis: String,
	pub flag_ipc_off: bool,
	pub flag_ipc_path: String,
	pub flag_ipc_apis: String,
	pub flag_dapps_off: bool,
	pub flag_dapps_port: u16,
	pub flag_dapps_interface: String,
	pub flag_dapps_user: Option<String>,
	pub flag_dapps_pass: Option<String>,
	pub flag_force_sealing: bool,
	pub flag_author: String,
	pub flag_usd_per_tx: String,
	pub flag_usd_per_eth: String,
	pub flag_gas_floor_target: String,
	pub flag_extra_data: Option<String>,
	pub flag_tx_limit: usize,
	pub flag_logging: Option<String>,
	pub flag_version: bool,
	pub flag_from: String,
	pub flag_to: String,
	pub flag_format: Option<String>,
	pub flag_jitvm: bool,
	pub flag_no_color: bool,
	// legacy...
	pub flag_geth: bool,
	pub flag_nodekey: Option<String>,
	pub flag_nodiscover: bool,
	pub flag_maxpeers: Option<usize>,
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
	pub flag_networkid: Option<String>,
	pub flag_ipcdisable: bool,
	pub flag_ipcpath: Option<String>,
	pub flag_ipcapi: Option<String>,
}

pub fn print_version() {
	println!("\
Parity
  version {}
Copyright 2015, 2016 Ethcore (UK) Limited
License GPLv3+: GNU GPL version 3 or later <http://gnu.org/licenses/gpl.html>.
This is free software: you are free to change and redistribute it.
There is NO WARRANTY, to the extent permitted by law.

By Wood/Paronyan/Kotewicz/Drwięga/Volf.\
", version());
}

