// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

#[macro_use]
mod usage;
use dir;

usage! {
	{
		// Commands
		cmd_daemon: bool,
		cmd_wallet: bool,
		cmd_account: bool,
		cmd_new: bool,
		cmd_list: bool,
		cmd_export: bool,
		cmd_blocks: bool,
		cmd_state: bool,
		cmd_import: bool,
		cmd_signer: bool,
		cmd_new_token: bool,
		cmd_sign: bool,
		cmd_reject: bool,
		cmd_snapshot: bool,
		cmd_restore: bool,
		cmd_ui: bool,
		cmd_dapp: bool,
		cmd_tools: bool,
		cmd_hash: bool,
		cmd_kill: bool,
		cmd_db: bool,

		// Arguments
		arg_pid_file: String,
		arg_file: Option<String>,
		arg_path: Vec<String>,
		arg_id: Option<usize>,

		// Flags
		// -- Legacy Options
		flag_geth: bool,
		flag_testnet: bool,
		flag_import_geth_keys: bool,
		flag_datadir: Option<String>,
		flag_networkid: Option<u64>,
		flag_peers: Option<u16>,
		flag_nodekey: Option<String>,
		flag_nodiscover: bool,
		flag_jsonrpc: bool,
		flag_jsonrpc_off: bool,
		flag_webapp: bool,
		flag_dapps_off: bool,
		flag_rpc: bool,
		flag_rpcaddr: Option<String>,
		flag_rpcport: Option<u16>,
		flag_rpcapi: Option<String>,
		flag_rpccorsdomain: Option<String>,
		flag_ipcdisable: bool,
		flag_ipc_off: bool,
		flag_ipcapi: Option<String>,
		flag_ipcpath: Option<String>,
		flag_gasprice: Option<String>,
		flag_etherbase: Option<String>,
		flag_extradata: Option<String>,
		flag_cache: Option<u32>,

		// -- Miscellaneous Options
		flag_version: bool,
		flag_no_config: bool,
	}
	{
		// -- Operating Options
		flag_mode: String = "last", or |c: &Config| otry!(c.parity).mode.clone(),
		flag_mode_timeout: u64 = 300u64, or |c: &Config| otry!(c.parity).mode_timeout.clone(),
		flag_mode_alarm: u64 = 3600u64, or |c: &Config| otry!(c.parity).mode_alarm.clone(),
		flag_auto_update: String = "critical", or |c: &Config| otry!(c.parity).auto_update.clone(),
		flag_release_track: String = "current", or |c: &Config| otry!(c.parity).release_track.clone(),
		flag_no_download: bool = false, or |c: &Config| otry!(c.parity).no_download.clone(),
		flag_no_consensus: bool = false, or |c: &Config| otry!(c.parity).no_consensus.clone(),
		flag_chain: String = "homestead", or |c: &Config| otry!(c.parity).chain.clone(),
		flag_keys_path: String = "$BASE/keys", or |c: &Config| otry!(c.parity).keys_path.clone(),
		flag_identity: String = "", or |c: &Config| otry!(c.parity).identity.clone(),

		// -- Account Options
		flag_unlock: Option<String> = None,
			or |c: &Config| otry!(c.account).unlock.as_ref().map(|vec| Some(vec.join(","))),
		flag_password: Vec<String> = Vec::new(),
			or |c: &Config| otry!(c.account).password.clone(),
		flag_keys_iterations: u32 = 10240u32,
			or |c: &Config| otry!(c.account).keys_iterations.clone(),
		flag_no_hardware_wallets: bool = false,
			or |c: &Config| otry!(c.account).disable_hardware.clone(),


		flag_force_ui: bool = false,
			or |c: &Config| otry!(c.ui).force.clone(),
		flag_no_ui: bool = false,
			or |c: &Config| otry!(c.ui).disable.clone(),
		flag_ui_port: u16 = 8180u16,
			or |c: &Config| otry!(c.ui).port.clone(),
		flag_ui_interface: String = "local",
			or |c: &Config| otry!(c.ui).interface.clone(),
		flag_ui_path: String = "$BASE/signer",
			or |c: &Config| otry!(c.ui).path.clone(),
		// NOTE [todr] For security reasons don't put this to config files
		flag_ui_no_validation: bool = false, or |_| None,

		// -- Networking Options
		flag_no_warp: bool = false,
			or |c: &Config| otry!(c.network).warp.clone().map(|w| !w),
		flag_port: u16 = 30303u16,
			or |c: &Config| otry!(c.network).port.clone(),
		flag_min_peers: u16 = 25u16,
			or |c: &Config| otry!(c.network).min_peers.clone(),
		flag_max_peers: u16 = 50u16,
			or |c: &Config| otry!(c.network).max_peers.clone(),
		flag_max_pending_peers: u16 = 64u16,
			or |c: &Config| otry!(c.network).max_pending_peers.clone(),
		flag_snapshot_peers: u16 = 0u16,
			or |c: &Config| otry!(c.network).snapshot_peers.clone(),
		flag_nat: String = "any",
			or |c: &Config| otry!(c.network).nat.clone(),
		flag_allow_ips: String = "all",
			or |c: &Config| otry!(c.network).allow_ips.clone(),
		flag_network_id: Option<u64> = None,
			or |c: &Config| otry!(c.network).id.clone().map(Some),
		flag_bootnodes: Option<String> = None,
			or |c: &Config| otry!(c.network).bootnodes.as_ref().map(|vec| Some(vec.join(","))),
		flag_no_discovery: bool = false,
			or |c: &Config| otry!(c.network).discovery.map(|d| !d).clone(),
		flag_node_key: Option<String> = None,
			or |c: &Config| otry!(c.network).node_key.clone().map(Some),
		flag_reserved_peers: Option<String> = None,
			or |c: &Config| otry!(c.network).reserved_peers.clone().map(Some),
		flag_reserved_only: bool = false,
			or |c: &Config| otry!(c.network).reserved_only.clone(),
		flag_no_ancient_blocks: bool = false, or |_| None,

		// -- API and Console Options
		// RPC
		flag_no_jsonrpc: bool = false,
			or |c: &Config| otry!(c.rpc).disable.clone(),
		flag_jsonrpc_port: u16 = 8545u16,
			or |c: &Config| otry!(c.rpc).port.clone(),
		flag_jsonrpc_interface: String  = "local",
			or |c: &Config| otry!(c.rpc).interface.clone(),
		flag_jsonrpc_cors: Option<String> = None,
			or |c: &Config| otry!(c.rpc).cors.clone().map(Some),
		flag_jsonrpc_apis: String = "web3,eth,net,parity,traces,rpc",
			or |c: &Config| otry!(c.rpc).apis.as_ref().map(|vec| vec.join(",")),
		flag_jsonrpc_hosts: String = "none",
			or |c: &Config| otry!(c.rpc).hosts.as_ref().map(|vec| vec.join(",")),

		// IPC
		flag_no_ipc: bool = false,
			or |c: &Config| otry!(c.ipc).disable.clone(),
		flag_ipc_path: String = "$BASE/jsonrpc.ipc",
			or |c: &Config| otry!(c.ipc).path.clone(),
		flag_ipc_apis: String = "web3,eth,net,parity,parity_accounts,traces,rpc",
			or |c: &Config| otry!(c.ipc).apis.as_ref().map(|vec| vec.join(",")),

		// DAPPS
		flag_no_dapps: bool = false,
			or |c: &Config| otry!(c.dapps).disable.clone(),
		flag_dapps_port: u16 = 8080u16,
			or |c: &Config| otry!(c.dapps).port.clone(),
		flag_dapps_interface: String = "local",
			or |c: &Config| otry!(c.dapps).interface.clone(),
		flag_dapps_hosts: String = "none",
			or |c: &Config| otry!(c.dapps).hosts.as_ref().map(|vec| vec.join(",")),
		flag_dapps_cors: Option<String> = None,
			or |c: &Config| otry!(c.dapps).cors.clone().map(Some),
		flag_dapps_path: String = "$BASE/dapps",
			or |c: &Config| otry!(c.dapps).path.clone(),
		flag_dapps_user: Option<String> = None,
			or |c: &Config| otry!(c.dapps).user.clone().map(Some),
		flag_dapps_pass: Option<String> = None,
			or |c: &Config| otry!(c.dapps).pass.clone().map(Some),
		flag_dapps_apis_all: bool = false, or |_| None,

		// Secret Store
		flag_no_secretstore: bool = false,
			or |c: &Config| otry!(c.secretstore).disable.clone(),
		flag_secretstore_port: u16 = 8082u16,
			or |c: &Config| otry!(c.secretstore).port.clone(),
		flag_secretstore_interface: String = "local",
			or |c: &Config| otry!(c.secretstore).interface.clone(),
		flag_secretstore_path: String = "$BASE/secretstore",
			or |c: &Config| otry!(c.secretstore).path.clone(),

		// IPFS
		flag_ipfs_api: bool = false,
			or |c: &Config| otry!(c.ipfs).enable.clone(),
		flag_ipfs_api_port: u16 = 5001u16,
			or |c: &Config| otry!(c.ipfs).port.clone(),
		flag_ipfs_api_interface: String = "local",
			or |c: &Config| otry!(c.ipfs).interface.clone(),
		flag_ipfs_api_cors: Option<String> = None,
			or |c: &Config| otry!(c.ipfs).cors.clone().map(Some),
		flag_ipfs_api_hosts: String = "none",
			or |c: &Config| otry!(c.ipfs).hosts.as_ref().map(|vec| vec.join(",")),

		// -- Sealing/Mining Options
		flag_author: Option<String> = None,
			or |c: &Config| otry!(c.mining).author.clone().map(Some),
		flag_engine_signer: Option<String> = None,
			or |c: &Config| otry!(c.mining).engine_signer.clone().map(Some),
		flag_force_sealing: bool = false,
			or |c: &Config| otry!(c.mining).force_sealing.clone(),
		flag_reseal_on_txs: String = "own",
			or |c: &Config| otry!(c.mining).reseal_on_txs.clone(),
		flag_reseal_min_period: u64 = 2000u64,
			or |c: &Config| otry!(c.mining).reseal_min_period.clone(),
		flag_work_queue_size: usize = 20usize,
			or |c: &Config| otry!(c.mining).work_queue_size.clone(),
		flag_tx_gas_limit: Option<String> = None,
			or |c: &Config| otry!(c.mining).tx_gas_limit.clone().map(Some),
		flag_tx_time_limit: Option<u64> = None,
			or |c: &Config| otry!(c.mining).tx_time_limit.clone().map(Some),
		flag_relay_set: String = "cheap",
			or |c: &Config| otry!(c.mining).relay_set.clone(),
		flag_usd_per_tx: String = "0.0025",
			or |c: &Config| otry!(c.mining).usd_per_tx.clone(),
		flag_usd_per_eth: String = "auto",
			or |c: &Config| otry!(c.mining).usd_per_eth.clone(),
		flag_price_update_period: String = "hourly",
			or |c: &Config| otry!(c.mining).price_update_period.clone(),
		flag_gas_floor_target: String = "4700000",
			or |c: &Config| otry!(c.mining).gas_floor_target.clone(),
		flag_gas_cap: String = "6283184",
			or |c: &Config| otry!(c.mining).gas_cap.clone(),
		flag_extra_data: Option<String> = None,
			or |c: &Config| otry!(c.mining).extra_data.clone().map(Some),
		flag_tx_queue_size: usize = 1024usize,
			or |c: &Config| otry!(c.mining).tx_queue_size.clone(),
		flag_tx_queue_gas: String = "auto",
			or |c: &Config| otry!(c.mining).tx_queue_gas.clone(),
		flag_tx_queue_strategy: String = "gas_price",
			or |c: &Config| otry!(c.mining).tx_queue_strategy.clone(),
		flag_tx_queue_ban_count: u16 = 1u16,
			or |c: &Config| otry!(c.mining).tx_queue_ban_count.clone(),
		flag_tx_queue_ban_time: u16 = 180u16,
			or |c: &Config| otry!(c.mining).tx_queue_ban_time.clone(),
		flag_remove_solved: bool = false,
			or |c: &Config| otry!(c.mining).remove_solved.clone(),
		flag_notify_work: Option<String> = None,
			or |c: &Config| otry!(c.mining).notify_work.as_ref().map(|vec| Some(vec.join(","))),
		flag_refuse_service_transactions: bool = false,
			or |c: &Config| otry!(c.mining).refuse_service_transactions.clone(),

		flag_stratum: bool = false,
			or |c: &Config| Some(c.stratum.is_some()),
		flag_stratum_interface: String = "local",
			or |c: &Config| otry!(c.stratum).interface.clone(),
		flag_stratum_port: u16 = 8008u16,
			or |c: &Config| otry!(c.stratum).port.clone(),
		flag_stratum_secret: Option<String> = None,
			or |c: &Config| otry!(c.stratum).secret.clone().map(Some),

		// -- Footprint Options
		flag_tracing: String = "auto",
			or |c: &Config| otry!(c.footprint).tracing.clone(),
		flag_pruning: String = "auto",
			or |c: &Config| otry!(c.footprint).pruning.clone(),
		flag_pruning_history: u64 = 64u64,
			or |c: &Config| otry!(c.footprint).pruning_history.clone(),
		flag_pruning_memory: usize = 75usize,
			or |c: &Config| otry!(c.footprint).pruning_memory.clone(),
		flag_cache_size_db: u32 = 64u32,
			or |c: &Config| otry!(c.footprint).cache_size_db.clone(),
		flag_cache_size_blocks: u32 = 8u32,
			or |c: &Config| otry!(c.footprint).cache_size_blocks.clone(),
		flag_cache_size_queue: u32 = 50u32,
			or |c: &Config| otry!(c.footprint).cache_size_queue.clone(),
		flag_cache_size_state: u32 = 25u32,
			or |c: &Config| otry!(c.footprint).cache_size_state.clone(),
		flag_cache_size: Option<u32> = None,
			or |c: &Config| otry!(c.footprint).cache_size.clone().map(Some),
		flag_fast_and_loose: bool = false,
			or |c: &Config| otry!(c.footprint).fast_and_loose.clone(),
		flag_db_compaction: String = "auto",
			or |c: &Config| otry!(c.footprint).db_compaction.clone(),
		flag_fat_db: String = "auto",
			or |c: &Config| otry!(c.footprint).fat_db.clone(),
		flag_scale_verifiers: bool = false,
			or |c: &Config| otry!(c.footprint).scale_verifiers.clone(),
		flag_num_verifiers: Option<usize> = None,
			or |c: &Config| otry!(c.footprint).num_verifiers.clone().map(Some),

		// -- Import/Export Options
		flag_from: String = "1", or |_| None,
		flag_to: String = "latest", or |_| None,
		flag_format: Option<String> = None, or |_| None,
		flag_no_seal_check: bool = false, or |_| None,
		flag_no_storage: bool = false, or |_| None,
		flag_no_code: bool = false, or |_| None,
		flag_min_balance: Option<String> = None, or |_| None,
		flag_max_balance: Option<String> = None, or |_| None,

		// -- Snapshot Optons
		flag_at: String = "latest", or |_| None,
		flag_no_periodic_snapshot: bool = false,
			or |c: &Config| otry!(c.snapshots).disable_periodic.clone(),

		// -- Virtual Machine Options
		flag_jitvm: bool = false,
			or |c: &Config| otry!(c.vm).jit.clone(),

		// -- Miscellaneous Options
		flag_config: String = "$BASE/config.toml", or |_| None,
		flag_logging: Option<String> = None,
			or |c: &Config| otry!(c.misc).logging.clone().map(Some),
		flag_log_file: Option<String> = None,
			or |c: &Config| otry!(c.misc).log_file.clone().map(Some),
		flag_no_color: bool = false,
			or |c: &Config| otry!(c.misc).color.map(|c| !c).clone(),
	}
	{
		// Values with optional default value.
		flag_base_path: Option<String>, display dir::default_data_path(), or |c: &Config| otry!(c.parity).base_path.clone().map(Some),
		flag_db_path: Option<String>, display dir::CHAINS_PATH, or |c: &Config| otry!(c.parity).db_path.clone().map(Some),
		flag_warp: Option<bool>, display true, or |c: &Config| Some(otry!(c.network).warp.clone()),
	}
}


#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Config {
	parity: Option<Operating>,
	account: Option<Account>,
	ui: Option<Ui>,
	network: Option<Network>,
	rpc: Option<Rpc>,
	ipc: Option<Ipc>,
	dapps: Option<Dapps>,
	secretstore: Option<SecretStore>,
	ipfs: Option<Ipfs>,
	mining: Option<Mining>,
	footprint: Option<Footprint>,
	snapshots: Option<Snapshots>,
	vm: Option<VM>,
	misc: Option<Misc>,
	stratum: Option<Stratum>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Operating {
	mode: Option<String>,
	mode_timeout: Option<u64>,
	mode_alarm: Option<u64>,
	auto_update: Option<String>,
	release_track: Option<String>,
	no_download: Option<bool>,
	no_consensus: Option<bool>,
	chain: Option<String>,
	base_path: Option<String>,
	db_path: Option<String>,
	keys_path: Option<String>,
	identity: Option<String>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Account {
	unlock: Option<Vec<String>>,
	password: Option<Vec<String>>,
	keys_iterations: Option<u32>,
	disable_hardware: Option<bool>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Ui {
	force: Option<bool>,
	disable: Option<bool>,
	port: Option<u16>,
	interface: Option<String>,
	path: Option<String>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Network {
	warp: Option<bool>,
	port: Option<u16>,
	min_peers: Option<u16>,
	max_peers: Option<u16>,
	snapshot_peers: Option<u16>,
	max_pending_peers: Option<u16>,
	nat: Option<String>,
	allow_ips: Option<String>,
	id: Option<u64>,
	bootnodes: Option<Vec<String>>,
	discovery: Option<bool>,
	node_key: Option<String>,
	reserved_peers: Option<String>,
	reserved_only: Option<bool>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Rpc {
	disable: Option<bool>,
	port: Option<u16>,
	interface: Option<String>,
	cors: Option<String>,
	apis: Option<Vec<String>>,
	hosts: Option<Vec<String>>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Ipc {
	disable: Option<bool>,
	path: Option<String>,
	apis: Option<Vec<String>>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Dapps {
	disable: Option<bool>,
	port: Option<u16>,
	interface: Option<String>,
	hosts: Option<Vec<String>>,
	cors: Option<String>,
	path: Option<String>,
	user: Option<String>,
	pass: Option<String>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct SecretStore {
	disable: Option<bool>,
	port: Option<u16>,
	interface: Option<String>,
	path: Option<String>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Ipfs {
	enable: Option<bool>,
	port: Option<u16>,
	interface: Option<String>,
	cors: Option<String>,
	hosts: Option<Vec<String>>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Mining {
	author: Option<String>,
	engine_signer: Option<String>,
	force_sealing: Option<bool>,
	reseal_on_txs: Option<String>,
	reseal_min_period: Option<u64>,
	work_queue_size: Option<usize>,
	tx_gas_limit: Option<String>,
	tx_time_limit: Option<u64>,
	relay_set: Option<String>,
	usd_per_tx: Option<String>,
	usd_per_eth: Option<String>,
	price_update_period: Option<String>,
	gas_floor_target: Option<String>,
	gas_cap: Option<String>,
	extra_data: Option<String>,
	tx_queue_size: Option<usize>,
	tx_queue_gas: Option<String>,
	tx_queue_strategy: Option<String>,
	tx_queue_ban_count: Option<u16>,
	tx_queue_ban_time: Option<u16>,
	remove_solved: Option<bool>,
	notify_work: Option<Vec<String>>,
	refuse_service_transactions: Option<bool>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Stratum {
	interface: Option<String>,
	port: Option<u16>,
	secret: Option<String>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Footprint {
	tracing: Option<String>,
	pruning: Option<String>,
	pruning_history: Option<u64>,
	pruning_memory: Option<usize>,
	fast_and_loose: Option<bool>,
	cache_size: Option<u32>,
	cache_size_db: Option<u32>,
	cache_size_blocks: Option<u32>,
	cache_size_queue: Option<u32>,
	cache_size_state: Option<u32>,
	db_compaction: Option<String>,
	fat_db: Option<String>,
	scale_verifiers: Option<bool>,
	num_verifiers: Option<usize>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Snapshots {
	disable_periodic: Option<bool>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct VM {
	jit: Option<bool>,
}

#[derive(Default, Debug, PartialEq, RustcDecodable)]
struct Misc {
	logging: Option<String>,
	log_file: Option<String>,
	color: Option<bool>,
}

#[cfg(test)]
mod tests {
	use super::{
		Args, ArgsError,
		Config, Operating, Account, Ui, Network, Rpc, Ipc, Dapps, Ipfs, Mining, Footprint,
		Snapshots, VM, Misc, SecretStore,
	};
	use toml;

	#[test]
	fn should_parse_args_and_include_config() {
		// given
		let mut config = Config::default();
		let mut operating = Operating::default();
		operating.chain = Some("morden".into());
		config.parity = Some(operating);

		// when
		let args = Args::parse_with_config(&["parity"], config).unwrap();

		// then
		assert_eq!(args.flag_chain, "morden".to_owned());
	}

	#[test]
	fn should_not_use_config_if_cli_is_provided() {
		// given
		let mut config = Config::default();
		let mut operating = Operating::default();
		operating.chain = Some("morden".into());
		config.parity = Some(operating);

		// when
		let args = Args::parse_with_config(&["parity", "--chain", "xyz"], config).unwrap();

		// then
		assert_eq!(args.flag_chain, "xyz".to_owned());
	}

	#[test]
	fn should_use_config_if_cli_is_missing() {
		let mut config = Config::default();
		let mut footprint = Footprint::default();
		footprint.pruning_history = Some(128);
		config.footprint = Some(footprint);

		// when
		let args = Args::parse_with_config(&["parity"], config).unwrap();

		// then
		assert_eq!(args.flag_pruning_history, 128);
	}

	#[test]
	fn should_parse_full_config() {
		// given
		let config = toml::decode_str(include_str!("./config.full.toml")).unwrap();

		// when
		let args = Args::parse_with_config(&["parity", "--chain", "xyz"], config).unwrap();

		// then
		assert_eq!(args, Args {
			// Commands
			cmd_daemon: false,
			cmd_wallet: false,
			cmd_account: false,
			cmd_new: false,
			cmd_list: false,
			cmd_export: false,
			cmd_state: false,
			cmd_blocks: false,
			cmd_import: false,
			cmd_signer: false,
			cmd_sign: false,
			cmd_reject: false,
			cmd_new_token: false,
			cmd_snapshot: false,
			cmd_restore: false,
			cmd_ui: false,
			cmd_dapp: false,
			cmd_tools: false,
			cmd_hash: false,
			cmd_db: false,
			cmd_kill: false,

			// Arguments
			arg_pid_file: "".into(),
			arg_file: None,
			arg_id: None,
			arg_path: vec![],

			// -- Operating Options
			flag_mode: "last".into(),
			flag_mode_timeout: 300u64,
			flag_mode_alarm: 3600u64,
			flag_auto_update: "none".into(),
			flag_release_track: "current".into(),
			flag_no_download: false,
			flag_no_consensus: false,
			flag_chain: "xyz".into(),
			flag_base_path: Some("$HOME/.parity".into()),
			flag_db_path: Some("$HOME/.parity/chains".into()),
			flag_keys_path: "$HOME/.parity/keys".into(),
			flag_identity: "".into(),

			// -- Account Options
			flag_unlock: Some("0xdeadbeefcafe0000000000000000000000000000".into()),
			flag_password: vec!["~/.safe/password.file".into()],
			flag_keys_iterations: 10240u32,
			flag_no_hardware_wallets: false,

			flag_force_ui: false,
			flag_no_ui: false,
			flag_ui_port: 8180u16,
			flag_ui_interface: "127.0.0.1".into(),
			flag_ui_path: "$HOME/.parity/signer".into(),
			flag_ui_no_validation: false,

			// -- Networking Options
			flag_no_warp: false,
			flag_port: 30303u16,
			flag_min_peers: 25u16,
			flag_max_peers: 50u16,
			flag_max_pending_peers: 64u16,
			flag_snapshot_peers: 0u16,
			flag_allow_ips: "all".into(),
			flag_nat: "any".into(),
			flag_network_id: Some(1),
			flag_bootnodes: Some("".into()),
			flag_no_discovery: false,
			flag_node_key: None,
			flag_reserved_peers: Some("./path_to_file".into()),
			flag_reserved_only: false,
			flag_no_ancient_blocks: false,

			// -- API and Console Options
			// RPC
			flag_no_jsonrpc: false,
			flag_jsonrpc_port: 8545u16,
			flag_jsonrpc_interface: "local".into(),
			flag_jsonrpc_cors: Some("null".into()),
			flag_jsonrpc_apis: "web3,eth,net,parity,traces,rpc".into(),
			flag_jsonrpc_hosts: "none".into(),

			// IPC
			flag_no_ipc: false,
			flag_ipc_path: "$HOME/.parity/jsonrpc.ipc".into(),
			flag_ipc_apis: "web3,eth,net,parity,parity_accounts,personal,traces,rpc".into(),

			// DAPPS
			flag_no_dapps: false,
			flag_dapps_port: 8080u16,
			flag_dapps_interface: "local".into(),
			flag_dapps_hosts: "none".into(),
			flag_dapps_cors: None,
			flag_dapps_path: "$HOME/.parity/dapps".into(),
			flag_dapps_user: Some("test_user".into()),
			flag_dapps_pass: Some("test_pass".into()),
			flag_dapps_apis_all: false,

			flag_no_secretstore: false,
			flag_secretstore_port: 8082u16,
			flag_secretstore_interface: "local".into(),
			flag_secretstore_path: "$HOME/.parity/secretstore".into(),

			// IPFS
			flag_ipfs_api: false,
			flag_ipfs_api_port: 5001u16,
			flag_ipfs_api_interface: "local".into(),
			flag_ipfs_api_cors: Some("null".into()),
			flag_ipfs_api_hosts: "none".into(),

			// -- Sealing/Mining Options
			flag_author: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
			flag_engine_signer: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
			flag_force_sealing: true,
			flag_reseal_on_txs: "all".into(),
			flag_reseal_min_period: 4000u64,
			flag_work_queue_size: 20usize,
			flag_tx_gas_limit: Some("6283184".into()),
			flag_tx_time_limit: Some(100u64),
			flag_relay_set: "cheap".into(),
			flag_usd_per_tx: "0.0025".into(),
			flag_usd_per_eth: "auto".into(),
			flag_price_update_period: "hourly".into(),
			flag_gas_floor_target: "4700000".into(),
			flag_gas_cap: "6283184".into(),
			flag_extra_data: Some("Parity".into()),
			flag_tx_queue_size: 1024usize,
			flag_tx_queue_gas: "auto".into(),
			flag_tx_queue_strategy: "gas_factor".into(),
			flag_tx_queue_ban_count: 1u16,
			flag_tx_queue_ban_time: 180u16,
			flag_remove_solved: false,
			flag_notify_work: Some("http://localhost:3001".into()),
			flag_refuse_service_transactions: false,

			flag_stratum: false,
			flag_stratum_interface: "local".to_owned(),
			flag_stratum_port: 8008u16,
			flag_stratum_secret: None,

			// -- Footprint Options
			flag_tracing: "auto".into(),
			flag_pruning: "auto".into(),
			flag_pruning_history: 64u64,
			flag_pruning_memory: 500usize,
			flag_cache_size_db: 64u32,
			flag_cache_size_blocks: 8u32,
			flag_cache_size_queue: 50u32,
			flag_cache_size_state: 25u32,
			flag_cache_size: Some(128),
			flag_fast_and_loose: false,
			flag_db_compaction: "ssd".into(),
			flag_fat_db: "auto".into(),
			flag_scale_verifiers: true,
			flag_num_verifiers: Some(6),

			// -- Import/Export Options
			flag_from: "1".into(),
			flag_to: "latest".into(),
			flag_format: None,
			flag_no_seal_check: false,
			flag_no_code: false,
			flag_no_storage: false,
			flag_min_balance: None,
			flag_max_balance: None,

			// -- Snapshot Optons
			flag_at: "latest".into(),
			flag_no_periodic_snapshot: false,

			// -- Virtual Machine Options
			flag_jitvm: false,

			// -- Legacy Options
			flag_geth: false,
			flag_testnet: false,
			flag_import_geth_keys: false,
			flag_datadir: None,
			flag_networkid: None,
			flag_peers: None,
			flag_nodekey: None,
			flag_nodiscover: false,
			flag_jsonrpc: false,
			flag_jsonrpc_off: false,
			flag_webapp: false,
			flag_dapps_off: false,
			flag_rpc: false,
			flag_rpcaddr: None,
			flag_rpcport: None,
			flag_rpcapi: None,
			flag_rpccorsdomain: None,
			flag_ipcdisable: false,
			flag_ipc_off: false,
			flag_ipcapi: None,
			flag_ipcpath: None,
			flag_gasprice: None,
			flag_etherbase: None,
			flag_extradata: None,
			flag_cache: None,
			flag_warp: Some(true),

			// -- Miscellaneous Options
			flag_version: false,
			flag_config: "$BASE/config.toml".into(),
			flag_logging: Some("own_tx=trace".into()),
			flag_log_file: Some("/var/log/parity.log".into()),
			flag_no_color: false,
			flag_no_config: false,
		});
	}

	#[test]
	fn should_parse_config_and_return_errors() {
		let config1 = Args::parse_config(include_str!("./config.invalid1.toml"));
		let config2 = Args::parse_config(include_str!("./config.invalid2.toml"));
		let config3 = Args::parse_config(include_str!("./config.invalid3.toml"));

		match (config1, config2, config3) {
			(Err(ArgsError::Parsing(_)), Err(ArgsError::Decode(_)), Err(ArgsError::UnknownFields(_))) => {},
			(a, b, c) => {
				assert!(false, "Got invalid error types: {:?}, {:?}, {:?}", a, b, c);
			}
		}
	}

	#[test]
	fn should_deserialize_toml_file() {
		let config: Config = toml::decode_str(include_str!("./config.toml")).unwrap();

		assert_eq!(config, Config {
			parity: Some(Operating {
				mode: Some("dark".into()),
				mode_timeout: Some(15u64),
				mode_alarm: Some(10u64),
				auto_update: None,
				release_track: None,
				no_download: None,
				no_consensus: None,
				chain: Some("./chain.json".into()),
				base_path: None,
				db_path: None,
				keys_path: None,
				identity: None,
			}),
			account: Some(Account {
				unlock: Some(vec!["0x1".into(), "0x2".into(), "0x3".into()]),
				password: Some(vec!["passwdfile path".into()]),
				keys_iterations: None,
				disable_hardware: None,
			}),
			ui: Some(Ui {
				force: None,
				disable: Some(true),
				port: None,
				interface: None,
				path: None,
			}),
			network: Some(Network {
				warp: Some(false),
				port: None,
				min_peers: Some(10),
				max_peers: Some(20),
				max_pending_peers: Some(30),
				snapshot_peers: Some(40),
				allow_ips: Some("public".into()),
				nat: Some("any".into()),
				id: None,
				bootnodes: None,
				discovery: Some(true),
				node_key: None,
				reserved_peers: Some("./path/to/reserved_peers".into()),
				reserved_only: Some(true),
			}),
			rpc: Some(Rpc {
				disable: Some(true),
				port: Some(8180),
				interface: None,
				cors: None,
				apis: None,
				hosts: None,
			}),
			ipc: Some(Ipc {
				disable: None,
				path: None,
				apis: Some(vec!["rpc".into(), "eth".into()]),
			}),
			dapps: Some(Dapps {
				disable: None,
				port: Some(8080),
				path: None,
				interface: None,
				hosts: None,
				cors: None,
				user: Some("username".into()),
				pass: Some("password".into())
			}),
			secretstore: Some(SecretStore {
				disable: None,
				port: Some(8082),
				interface: None,
				path: None,
			}),
			ipfs: Some(Ipfs {
				enable: Some(false),
				port: Some(5001),
				interface: None,
				cors: None,
				hosts: None,
			}),
			mining: Some(Mining {
				author: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
				engine_signer: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
				force_sealing: Some(true),
				reseal_on_txs: Some("all".into()),
				reseal_min_period: Some(4000),
				work_queue_size: None,
				relay_set: None,
				usd_per_tx: None,
				usd_per_eth: None,
				price_update_period: Some("hourly".into()),
				gas_floor_target: None,
				gas_cap: None,
				tx_queue_size: Some(1024),
				tx_queue_gas: Some("auto".into()),
				tx_queue_strategy: None,
				tx_queue_ban_count: None,
				tx_queue_ban_time: None,
				tx_gas_limit: None,
				tx_time_limit: None,
				extra_data: None,
				remove_solved: None,
				notify_work: None,
				refuse_service_transactions: None,
			}),
			footprint: Some(Footprint {
				tracing: Some("on".into()),
				pruning: Some("fast".into()),
				pruning_history: Some(64),
				pruning_memory: None,
				fast_and_loose: None,
				cache_size: None,
				cache_size_db: Some(128),
				cache_size_blocks: Some(16),
				cache_size_queue: Some(100),
				cache_size_state: Some(25),
				db_compaction: Some("ssd".into()),
				fat_db: Some("off".into()),
				scale_verifiers: Some(false),
				num_verifiers: None,
			}),
			snapshots: Some(Snapshots {
				disable_periodic: Some(true),
			}),
			vm: Some(VM {
				jit: Some(false),
			}),
			misc: Some(Misc {
				logging: Some("own_tx=trace".into()),
				log_file: Some("/var/log/parity.log".into()),
				color: Some(true),
			}),
			stratum: None,
		});
	}
}
