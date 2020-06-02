use clap::Clap;
use cli::config::get_config;
use cli::globals::{Globals, IPCOptions};
use cli::parse_cli::*;
use cli::subcommands::*;
use std::fs;

#[derive(Debug, PartialEq)]
pub enum ArgsError {
	ConfigParseError(String),
	ConfigReadError(String),
	ConfigWriteError(String),
	PeerConfigurationError(String),
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Args {
	pub cmd_daemon: bool,
	pub arg_daemon_pid_file: Option<String>,

	pub cmd_account: bool,
	pub cmd_account_new: bool,
	pub cmd_account_list: bool,
	pub cmd_account_import: bool,
	pub arg_account_import_path: Option<Vec<String>>,

	pub cmd_wallet: bool,
	pub cmd_wallet_import: bool,
	pub arg_wallet_import_path: Option<String>,

	pub cmd_import: bool,
	pub arg_import_format: Option<String>,
	pub arg_import_file: Option<String>,

	pub cmd_export: bool,
	pub cmd_export_blocks: bool,
	pub arg_export_blocks_format: Option<String>,
	pub arg_export_blocks_from: String,
	pub arg_export_blocks_to: String,
	pub arg_export_blocks_file: Option<String>,

	pub cmd_export_state: bool,
	pub flag_export_state_no_storage: bool,
	pub flag_export_state_no_code: bool,
	pub arg_export_state_min_balance: Option<String>,
	pub arg_export_state_max_balance: Option<String>,
	pub arg_export_state_at: String,
	pub arg_export_state_format: Option<String>,
	pub arg_export_state_file: Option<String>,

	pub cmd_signer: bool,
	pub cmd_signer_new_token: bool,
	pub cmd_signer_list: bool,
	pub cmd_signer_sign: bool,
	pub arg_signer_sign_id: Option<usize>,
	pub cmd_signer_reject: bool,
	pub arg_signer_reject_id: Option<usize>,

	pub cmd_snapshot: bool,
	pub arg_snapshot_at: String,
	pub arg_snapshot_file: Option<String>,

	pub cmd_restore: bool,
	pub arg_restore_file: Option<String>,

	pub cmd_tools: bool,
	pub cmd_tools_hash: bool,
	pub arg_tools_hash_file: Option<String>,

	pub cmd_db: bool,
	pub cmd_db_kill: bool,
	pub cmd_db_reset: bool,
	pub arg_db_reset_num: u32,

	pub cmd_export_hardcoded_sync: bool,

	pub cmd_dapp: bool,
	pub arg_dapp_path: Option<String>,

	pub flag_no_download: bool,
	pub flag_no_consensus: bool,
	pub flag_light: bool,
	pub flag_no_hardcoded_sync: bool,
	pub flag_force_direct: bool,
	pub arg_mode: String,
	pub arg_mode_timeout: u64,
	pub arg_mode_alarm: u64,
	pub arg_auto_update: String,
	pub arg_auto_update_delay: u16,
	pub arg_auto_update_check_frequency: u16,
	pub arg_release_track: String,
	pub arg_chain: String,
	pub arg_keys_path: String,
	pub arg_identity: String,
	pub arg_base_path: Option<String>,
	pub arg_db_path: Option<String>,
	pub arg_sync_until: Option<u64>,
	pub flag_unsafe_expose: bool,
	pub arg_config: String,
	pub arg_ports_shift: u16,
	pub flag_fast_unlock: bool,
	pub arg_keys_iterations: u32,
	pub arg_accounts_refresh: u64,
	pub arg_unlock: Option<String>,
	pub arg_enable_signing_queue: bool,
	pub arg_password: Vec<String>,
	pub flag_private_enabled: bool,
	pub flag_private_state_offchain: bool,
	pub arg_private_signer: Option<String>,
	pub arg_private_validators: Option<String>,
	pub arg_private_account: Option<String>,
	pub arg_private_sstore_url: Option<String>,
	pub arg_private_sstore_threshold: Option<u32>,
	pub arg_private_passwords: Option<String>,
	pub arg_ui_path: String,
	pub flag_no_warp: bool,
	pub flag_no_discovery: bool,
	pub flag_reserved_only: bool,
	pub flag_no_ancient_blocks: bool,
	pub flag_no_serve_light: bool,
	pub arg_warp_barrier: Option<u64>,
	pub arg_port: u16,
	pub arg_interface: String,
	pub arg_min_peers: Option<u16>,
	pub arg_max_peers: Option<u16>,
	pub arg_snapshot_peers: u16,
	pub arg_nat: String,
	pub arg_allow_ips: String,
	pub arg_max_pending_peers: u16,
	pub arg_network_id: Option<u64>,
	pub arg_bootnodes: Option<String>,
	pub arg_node_key: Option<String>,
	pub arg_reserved_peers: Option<String>,
	pub flag_jsonrpc_allow_missing_blocks: bool,
	pub flag_no_jsonrpc: bool,
	pub flag_jsonrpc_no_keep_alive: bool,
	pub flag_jsonrpc_experimental: bool,
	pub arg_jsonrpc_port: u16,
	pub arg_jsonrpc_interface: String,
	pub arg_jsonrpc_apis: String,
	pub arg_jsonrpc_hosts: String,
	pub arg_jsonrpc_server_threads: Option<usize>,
	pub arg_jsonrpc_cors: String,
	pub arg_jsonrpc_max_payload: Option<usize>,
	pub arg_poll_lifetime: u32,
	pub flag_no_ws: bool,
	pub arg_ws_port: u16,
	pub arg_ws_interface: String,
	pub arg_ws_apis: String,
	pub arg_ws_origins: String,
	pub arg_ws_hosts: String,
	pub arg_ws_max_connections: usize,
	pub flag_no_ipc: bool,
	pub arg_ipc_path: String,
	pub arg_ipc_chmod: String,
	pub arg_ipc_apis: String,
	pub arg_on_demand_response_time_window: Option<u64>,
	pub arg_on_demand_request_backoff_start: Option<u64>,
	pub arg_on_demand_request_backoff_max: Option<u64>,
	pub arg_on_demand_request_backoff_rounds_max: Option<usize>,
	pub arg_on_demand_request_consecutive_failures: Option<usize>,
	pub flag_no_secretstore: bool,
	pub flag_no_secretstore_http: bool,
	pub flag_no_secretstore_auto_migrate: bool,
	pub arg_secretstore_http_cors: String,
	pub arg_secretstore_acl_contract: Option<String>,
	pub arg_secretstore_contract: Option<String>,
	pub arg_secretstore_srv_gen_contract: Option<String>,
	pub arg_secretstore_srv_retr_contract: Option<String>,
	pub arg_secretstore_doc_store_contract: Option<String>,
	pub arg_secretstore_doc_sretr_contract: Option<String>,
	pub arg_secretstore_nodes: String,
	pub arg_secretstore_server_set_contract: Option<String>,
	pub arg_secretstore_interface: String,
	pub arg_secretstore_port: u16,
	pub arg_secretstore_http_interface: String,
	pub arg_secretstore_http_port: u16,
	pub arg_secretstore_path: String,
	pub arg_secretstore_secret: Option<String>,
	pub arg_secretstore_admin_public: Option<String>,
	pub flag_force_sealing: bool,
	pub flag_reseal_on_uncle: bool,
	pub flag_remove_solved: bool,
	pub flag_tx_queue_no_unfamiliar_locals: bool,
	pub flag_tx_queue_no_early_reject: bool,
	pub flag_refuse_service_transactions: bool,
	pub flag_infinite_pending_block: bool,
	pub flag_no_persistent_txqueue: bool,
	pub flag_stratum: bool,
	pub arg_reseal_on_txs: String,
	pub arg_reseal_min_period: u64,
	pub arg_reseal_max_period: u64,
	pub arg_work_queue_size: usize,
	pub arg_relay_set: String,
	pub arg_usd_per_tx: String,
	pub arg_usd_per_eth: String,
	pub arg_price_update_period: String,
	pub arg_gas_floor_target: String,
	pub arg_gas_cap: String,
	pub arg_tx_queue_mem_limit: u32,
	pub arg_tx_queue_size: usize,
	pub arg_tx_queue_per_sender: Option<usize>,
	pub arg_tx_queue_locals: Option<String>,
	pub arg_tx_queue_strategy: String,
	pub arg_stratum_interface: String,
	pub arg_stratum_port: u16,
	pub arg_min_gas_price: Option<u64>,
	pub arg_gas_price_percentile: usize,
	pub arg_author: Option<String>,
	pub arg_engine_signer: Option<String>,
	pub arg_tx_gas_limit: Option<String>,
	pub arg_tx_time_limit: Option<u64>,
	pub arg_extra_data: Option<String>,
	pub arg_notify_work: Option<String>,
	pub arg_stratum_secret: Option<String>,
	pub arg_max_round_blocks_to_import: usize,
	pub flag_can_restart: bool,
	pub flag_no_color: bool,
	pub flag_no_config: bool,
	pub arg_logging: Option<String>,
	pub arg_log_file: Option<String>,
	pub flag_scale_verifiers: bool,
	pub arg_tracing: String,
	pub arg_pruning: String,
	pub arg_pruning_history: u64,
	pub arg_pruning_memory: usize,
	pub arg_cache_size_db: u32,
	pub arg_cache_size_blocks: u32,
	pub arg_cache_size_queue: u32,
	pub arg_cache_size_state: u32,
	pub arg_db_compaction: String,
	pub arg_fat_db: String,
	pub arg_cache_size: Option<u32>,
	pub arg_num_verifiers: Option<usize>,
	pub flag_no_seal_check: bool,
	pub flag_no_periodic_snapshot: bool,
	pub arg_snapshot_threads: Option<usize>,
	pub flag_whisper: bool,
	pub arg_whisper_pool_size: Option<usize>,
	pub flag_geth: bool,
	pub flag_import_geth_keys: bool,
}

impl Args {
	pub fn parse() -> Result<Self, ArgsError> {
		let mut args: Args = Default::default();

		let mut raw_input = ArgsInput::parse();

		Args::save_current_config(&mut raw_input.globals)?;

		// This is the hardcoded config provided by openethereum, with
		// no special presets
		// FIXME: convert this to relative pathing
		let fallback_config_path = "config_default.toml";

		let default_config_path = match &raw_input.globals.convenience.config {
			// the has given a custom configuration
			Some(x) => {
				match x.as_str() {
					// presets
					"dev" => "config_dev.toml",
					"dev-insecure" => "config_dev_insecure.toml",
					"insecure" => "config_insecure.toml",
					"mining" => "config_mining.toml",
					"non-standard-ports" => "config_non_standard_ports.toml",

					// user given config path
					_ => x,
				}
			}

			// no input options for configuration
			None => {
				// We can make the default and the fallback the same
				"config_default.toml"
			}
		};

		let (default_config, fallback_config) =
			Args::generate_default_configuration(default_config_path, fallback_config_path)?;

		args.absorb_cli(raw_input, default_config, fallback_config)?;

		Ok(args)
	}

	pub fn save_current_config(globals: &mut Globals) -> Result<(), ArgsError> {
		// We don't want to print the config generate option on the generated toml, so we
		// reset it
		let config_generate = globals.convenience.config_generate.clone();
		globals.convenience.config_generate = None;

		if let Some(path) = &config_generate {
			let current_flags = toml::to_string(globals).map_err(|e| {
				ArgsError::ConfigWriteError(format!(
					"Failed to generate valid config toml from current flags: {}.Please report a bug if this error persists.", e
				))
			})?;

			fs::write(&path, current_flags).map_err(|_|  {
				ArgsError::ConfigParseError(format!(
					"Failed to write config to given file path {}. Please try again with a valid path and config name.", &path
				))
			})?;
		}
		Ok(())
	}

	pub fn generate_default_configuration(
		default_config_path: &str,
		fallback_config_path: &str,
	) -> Result<(Globals, Globals), ArgsError> {
		// FIXME: throw an error when file is not found
		let default_config_file:String = get_config(default_config_path)?;
		let fallback_config_file:String = get_config(fallback_config_path)?;

		let default_config: Globals = toml::from_str(&default_config_file).map_err(|e| {
				ArgsError::ConfigParseError(format!(
				"Failure to parse config file: {}, error: {}",
				default_config_path, e
			))
		})?;

		let fallback_config: Globals = toml::from_str(&fallback_config_file).map_err(|e|  {
				ArgsError::ConfigParseError(format!(
				"Failure to parse config file {}: {}",
				fallback_config_path, e
			))
		})?;

		Ok((default_config, fallback_config))
	}

	pub fn absorb_cli(
		&mut self,
		cli_args: ArgsInput,
		default_globals: Globals,
		fallback_globals: Globals,
	) -> Result<(), ArgsError> {
		self.absorb_subcommands(&cli_args)?;
		self.absorb_globals(cli_args, default_globals, fallback_globals)?;
		Ok(())
	}

	fn absorb_subcommands(&mut self, cli_args: &ArgsInput) -> Result<(), ArgsError> {
		match &cli_args.subcommands {
			None => {}
			Some(subcommand) => match &subcommand {
				SubCommands::Daemon(d) => {
					self.cmd_daemon = true;

					self.arg_daemon_pid_file = d.pid_file.clone();
				}
				SubCommands::Wallet { wallet } => {
					self.cmd_wallet = true;

					let Wallet::Import { path } = wallet;
					self.cmd_wallet_import = true;
					self.arg_wallet_import_path = (*path).clone();
				}
				SubCommands::Account { account } => {
					self.cmd_account = true;

					match account {
						Account::New => {
							self.cmd_account_new = true;
						}
						Account::Import { path } => {
							self.cmd_account_import = true;
							self.arg_account_import_path = Some((*path).clone());
						}
						Account::List => {
							self.cmd_account_list = true;
						}
					}
				}
				SubCommands::Import(i) => {
					self.cmd_import = true;
					self.arg_import_format = i.format.clone();
					self.arg_import_file = i.file.clone();
				}
				SubCommands::Export { export } => {
					self.cmd_export = true;
					match export {
						Export::Blocks(eb) => {
							self.cmd_export_blocks = true;
							self.arg_export_blocks_format = eb.format.clone();
							self.arg_export_blocks_from = eb.from.clone();
							self.arg_export_blocks_to = eb.to.clone();
							self.arg_export_blocks_file = eb.file.clone();
						}
						Export::State(es) => {
							self.cmd_export_state = true;
							self.flag_export_state_no_storage = es.no_storage;
							self.flag_export_state_no_code = es.no_code;
							self.arg_export_state_min_balance = es.min_balance.clone();
							self.arg_export_state_max_balance = es.max_balance.clone();
							self.arg_export_state_at = es.at.clone();
							self.arg_export_state_format = es.format.clone();
							self.arg_export_state_file = es.file.clone();
						}
					}
				}
				SubCommands::Signer(s) => {
					self.cmd_signer = true;
					match s {
						Signer::NewToken => {
							self.cmd_signer_new_token = true;
						}
						Signer::List => {
							self.cmd_signer_list = true;
						}
						Signer::Sign { id } => {
							self.cmd_signer_sign = true;
							self.arg_signer_sign_id = *id;
						}
						Signer::Reject { id } => {
							self.cmd_signer_reject = true;
							self.arg_signer_reject_id = *id;
						}
					}
				}
				SubCommands::Restore(r) => {
					self.cmd_restore = true;
					self.arg_restore_file = r.file.clone();
				}
				SubCommands::Snapshots(s) => {
					self.cmd_snapshot = true;
					self.arg_snapshot_at = s.at.clone();
					self.arg_snapshot_file = s.file.clone();
				}
				SubCommands::Db(db) => {
					self.cmd_db = true;
					match db {
						Db::Kill => {
							self.cmd_db_kill = true;
						}
						Db::Reset { num } => {
							self.cmd_db_reset = true;
							self.arg_db_reset_num = *num;
						}
					}
				}
				SubCommands::ExportHardcodedSync => {
					self.cmd_export_hardcoded_sync = true;
				}

			},
		}
		Ok(())
	}

	fn select_value<T>(raw: Option<T>, default: Option<T>, fallback: Option<T>) -> T {
				raw.or(default).or(fallback).expect("Value is always present, at least in fallback")
	}

	fn select_option<T>(raw: Option<T>, default: Option<T>, fallback: Option<T>) -> Option<T> {
		raw.or(default).or(fallback)
	}

	fn absorb_globals(
		&mut self,
		cli_args: ArgsInput,
		defaults: Globals,
		fallback: Globals,
	) -> Result<(), ArgsError> {
		self.absorb_global_flags(&cli_args, &defaults, &fallback);
		self.absorb_global_options(cli_args, defaults, fallback);

		if let (Some(min_peers), Some(max_peers)) = (self.arg_min_peers, self.arg_max_peers) {
			if max_peers < min_peers {
				return Err(ArgsError::PeerConfigurationError(
					"max-peers need to be greater than or equal to min-peers".to_owned(),
				));
			}
		}
		Ok(())
	}

	fn absorb_global_options(&mut self, cli_args: ArgsInput, defaults: Globals, fallback: Globals) {
		// Unnatural cases

		self.arg_ipc_path = Args::select_value(
			cli_args.globals.ipc.ipc_path,
			Some(IPCOptions::ipc_path_default()),
			None, // We don't care about fallback in this case, since the previous operation is infallible
		);
		self.arg_password = cli_args.globals.account.password;

		self.arg_config = Args::select_value(
			cli_args.globals.convenience.config,
			defaults.convenience.config,
			fallback.convenience.config,
		);
		self.arg_ports_shift = Args::select_value(
			cli_args.globals.convenience.ports_shift,
			defaults.convenience.ports_shift,
			fallback.convenience.ports_shift,
		);
		self.arg_keys_iterations = Args::select_value(
			cli_args.globals.account.keys_iterations,
			defaults.account.keys_iterations,
			fallback.account.keys_iterations,
		);
		self.arg_accounts_refresh = Args::select_value(
			cli_args.globals.account.accounts_refresh,
			defaults.account.accounts_refresh,
			fallback.account.accounts_refresh,
		);
		self.arg_unlock = Args::select_option(
			cli_args.globals.account.unlock,
			defaults.account.unlock,
			fallback.account.unlock,
		);
		self.arg_db_path = Args::select_option(
			cli_args.globals.operating.db_path,
			defaults.operating.db_path,
			fallback.operating.db_path,
		);
		self.arg_sync_until = Args::select_option(
			cli_args.globals.operating.sync_until,
			defaults.operating.sync_until,
			fallback.operating.sync_until,
		);
		self.arg_mode = Args::select_value(
			cli_args.globals.operating.mode,
			defaults.operating.mode,
			fallback.operating.mode,
		);
		self.arg_mode_timeout = Args::select_value(
			cli_args.globals.operating.mode_timeout,
			defaults.operating.mode_timeout,
			fallback.operating.mode_timeout,
		);
		self.arg_mode_alarm = Args::select_value(
			cli_args.globals.operating.mode_alarm,
			defaults.operating.mode_alarm,
			fallback.operating.mode_alarm,
		);
		self.arg_auto_update = Args::select_value(
			cli_args.globals.operating.auto_update,
			defaults.operating.auto_update,
			fallback.operating.auto_update,
		);
		self.arg_auto_update_delay = Args::select_value(
			cli_args.globals.operating.auto_update_delay,
			defaults.operating.auto_update_delay,
			fallback.operating.auto_update_delay,
		);
		self.arg_auto_update_check_frequency = Args::select_value(
			cli_args.globals.operating.auto_update_check_frequency,
			defaults.operating.auto_update_check_frequency,
			fallback.operating.auto_update_check_frequency,
		);
		self.arg_release_track = Args::select_value(
			cli_args.globals.operating.release_track,
			defaults.operating.release_track,
			fallback.operating.release_track,
		);
		self.arg_chain = Args::select_value(
			cli_args.globals.operating.chain,
			defaults.operating.chain,
			fallback.operating.chain,
		);
		self.arg_keys_path = Args::select_value(
			cli_args.globals.operating.keys_path,
			defaults.operating.keys_path,
			fallback.operating.keys_path,
		);
		self.arg_identity = Args::select_value(
			cli_args.globals.operating.identity,
			defaults.operating.identity,
			fallback.operating.identity,
		);
		self.arg_base_path = Args::select_option(
			cli_args.globals.operating.base_path,
			defaults.operating.base_path,
			fallback.operating.base_path,
		);
		self.arg_private_validators = Args::select_option(
			cli_args.globals.private_transactions.private_validators,
			defaults.private_transactions.private_validators,
			fallback.private_transactions.private_validators,
		);
		self.arg_private_account = Args::select_option(
			cli_args.globals.private_transactions.private_account,
			defaults.private_transactions.private_account,
			fallback.private_transactions.private_account,
		);
		self.arg_private_sstore_url = Args::select_option(
			cli_args.globals.private_transactions.private_sstore_url,
			defaults.private_transactions.private_sstore_url,
			fallback.private_transactions.private_sstore_url,
		);
		self.arg_private_sstore_threshold = Args::select_option(
			cli_args
				.globals
				.private_transactions
				.private_sstore_threshold,
			defaults.private_transactions.private_sstore_threshold,
			fallback.private_transactions.private_sstore_threshold,
		);
		self.arg_private_passwords = Args::select_option(
			cli_args.globals.private_transactions.private_passwords,
			defaults.private_transactions.private_passwords,
			fallback.private_transactions.private_passwords,
		);
		self.arg_ui_path = Args::select_value(
			cli_args.globals.ui_options.ui_path,
			defaults.ui_options.ui_path,
			fallback.ui_options.ui_path,
		);
		self.arg_warp_barrier = Args::select_option(
			cli_args.globals.networking.warp_barrier,
			defaults.networking.warp_barrier,
			fallback.networking.warp_barrier,
		);
		self.arg_port = Args::select_value(
			cli_args.globals.networking.port,
			defaults.networking.port,
			fallback.networking.port,
		);
		self.arg_interface = Args::select_value(
			cli_args.globals.networking.interface,
			defaults.networking.interface,
			fallback.networking.interface,
		);
		self.arg_min_peers = Args::select_option(
			cli_args.globals.networking.min_peers,
			defaults.networking.min_peers,
			fallback.networking.min_peers,
		);
		self.arg_max_peers = Args::select_option(
			cli_args.globals.networking.max_peers,
			defaults.networking.max_peers,
			fallback.networking.max_peers,
		);
		self.arg_snapshot_peers = Args::select_value(
			cli_args.globals.networking.snapshot_peers,
			defaults.networking.snapshot_peers,
			fallback.networking.snapshot_peers,
		);
		self.arg_nat = Args::select_value(
			cli_args.globals.networking.nat,
			defaults.networking.nat,
			fallback.networking.nat,
		);
		self.arg_allow_ips = Args::select_value(
			cli_args.globals.networking.allow_ips,
			defaults.networking.allow_ips,
			fallback.networking.allow_ips,
		);
		self.arg_max_pending_peers = Args::select_value(
			cli_args.globals.networking.max_pending_peers,
			defaults.networking.max_pending_peers,
			fallback.networking.max_pending_peers,
		);
		self.arg_network_id = Args::select_option(
			cli_args.globals.networking.network_id,
			defaults.networking.network_id,
			fallback.networking.network_id,
		);
		self.arg_bootnodes = Args::select_option(
			cli_args.globals.networking.bootnodes,
			defaults.networking.bootnodes,
			fallback.networking.bootnodes,
		);
		self.arg_node_key = Args::select_option(
			cli_args.globals.networking.node_key,
			defaults.networking.node_key,
			fallback.networking.node_key,
		);
		self.arg_reserved_peers = Args::select_option(
			cli_args.globals.networking.reserved_peers,
			defaults.networking.reserved_peers,
			fallback.networking.reserved_peers,
		);
		self.arg_jsonrpc_port = Args::select_value(
			cli_args.globals.http_json_rpc.jsonrpc_port,
			defaults.http_json_rpc.jsonrpc_port,
			fallback.http_json_rpc.jsonrpc_port,
		);
		self.arg_jsonrpc_interface = Args::select_value(
			cli_args.globals.http_json_rpc.jsonrpc_interface,
			defaults.http_json_rpc.jsonrpc_interface,
			fallback.http_json_rpc.jsonrpc_interface,
		);
		self.arg_jsonrpc_apis = Args::select_value(
			cli_args.globals.http_json_rpc.jsonrpc_apis,
			defaults.http_json_rpc.jsonrpc_apis,
			fallback.http_json_rpc.jsonrpc_apis,
		);
		self.arg_jsonrpc_hosts = Args::select_value(
			cli_args.globals.http_json_rpc.jsonrpc_hosts,
			defaults.http_json_rpc.jsonrpc_hosts,
			fallback.http_json_rpc.jsonrpc_hosts,
		);
		self.arg_jsonrpc_server_threads = Args::select_option(
			cli_args.globals.http_json_rpc.jsonrpc_server_threads,
			defaults.http_json_rpc.jsonrpc_server_threads,
			fallback.http_json_rpc.jsonrpc_server_threads,
		);
		self.arg_jsonrpc_cors = Args::select_value(
			cli_args.globals.http_json_rpc.jsonrpc_cors,
			defaults.http_json_rpc.jsonrpc_cors,
			fallback.http_json_rpc.jsonrpc_cors,
		);
		self.arg_jsonrpc_max_payload = Args::select_option(
			cli_args.globals.http_json_rpc.jsonrpc_max_payload,
			defaults.http_json_rpc.jsonrpc_max_payload,
			fallback.http_json_rpc.jsonrpc_max_payload,
		);
		self.arg_poll_lifetime = Args::select_value(
			cli_args.globals.http_json_rpc.poll_lifetime,
			defaults.http_json_rpc.poll_lifetime,
			fallback.http_json_rpc.poll_lifetime,
		);
		self.arg_ws_port = Args::select_value(
			cli_args.globals.websockets.ws_port,
			defaults.websockets.ws_port,
			fallback.websockets.ws_port,
		);
		self.arg_ws_interface = Args::select_value(
			cli_args.globals.websockets.ws_interface,
			defaults.websockets.ws_interface,
			fallback.websockets.ws_interface,
		);
		self.arg_ws_apis = Args::select_value(
			cli_args.globals.websockets.ws_apis,
			defaults.websockets.ws_apis,
			fallback.websockets.ws_apis,
		);
		self.arg_ws_origins = Args::select_value(
			cli_args.globals.websockets.ws_origins,
			defaults.websockets.ws_origins,
			fallback.websockets.ws_origins,
		);
		self.arg_ws_hosts = Args::select_value(
			cli_args.globals.websockets.ws_hosts,
			defaults.websockets.ws_hosts,
			fallback.websockets.ws_hosts,
		);
		self.arg_ws_max_connections = Args::select_value(
			cli_args.globals.websockets.ws_max_connections,
			defaults.websockets.ws_max_connections,
			fallback.websockets.ws_max_connections,
		);
		self.arg_ipc_chmod = Args::select_value(
			cli_args.globals.ipc.ipc_chmod,
			defaults.ipc.ipc_chmod,
			fallback.ipc.ipc_chmod,
		);
		self.arg_ipc_apis = Args::select_value(
			cli_args.globals.ipc.ipc_apis,
			defaults.ipc.ipc_apis,
			fallback.ipc.ipc_apis,
		);
		self.arg_on_demand_response_time_window = Args::select_option(
			cli_args.globals.light_client.on_demand_response_time_window,
			defaults.light_client.on_demand_response_time_window,
			fallback.light_client.on_demand_response_time_window,
		);
		self.arg_secretstore_http_cors = Args::select_value(
			cli_args.globals.secret_store.secretstore_http_cors,
			defaults.secret_store.secretstore_http_cors,
			fallback.secret_store.secretstore_http_cors,
		);
		self.arg_secretstore_acl_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_acl_contract,
			defaults.secret_store.secretstore_acl_contract,
			fallback.secret_store.secretstore_acl_contract,
		);
		self.arg_secretstore_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_contract,
			defaults.secret_store.secretstore_contract,
			fallback.secret_store.secretstore_contract,
		);
		self.arg_secretstore_interface = Args::select_value(
			cli_args.globals.secret_store.secretstore_interface,
			defaults.secret_store.secretstore_interface,
			fallback.secret_store.secretstore_interface,
		);
		self.arg_secretstore_port = Args::select_value(
			cli_args.globals.secret_store.secretstore_port,
			defaults.secret_store.secretstore_port,
			fallback.secret_store.secretstore_port,
		);
		self.arg_secretstore_http_port = Args::select_value(
			cli_args.globals.secret_store.secretstore_http_port,
			defaults.secret_store.secretstore_http_port,
			fallback.secret_store.secretstore_http_port,
		);
		self.arg_secretstore_path = Args::select_value(
			cli_args.globals.secret_store.secretstore_path,
			defaults.secret_store.secretstore_path,
			fallback.secret_store.secretstore_path,
		);
		self.arg_secretstore_secret = Args::select_option(
			cli_args.globals.secret_store.secretstore_secret,
			defaults.secret_store.secretstore_secret,
			fallback.secret_store.secretstore_secret,
		);
		self.arg_secretstore_admin_public = Args::select_option(
			cli_args.globals.secret_store.secretstore_admin_public,
			defaults.secret_store.secretstore_admin_public,
			fallback.secret_store.secretstore_admin_public,
		);
		self.arg_reseal_on_txs = Args::select_value(
			cli_args.globals.sealing_mining.reseal_on_txs,
			defaults.sealing_mining.reseal_on_txs,
			fallback.sealing_mining.reseal_on_txs,
		);
		self.arg_reseal_min_period = Args::select_value(
			cli_args.globals.sealing_mining.reseal_min_period,
			defaults.sealing_mining.reseal_min_period,
			fallback.sealing_mining.reseal_min_period,
		);
		self.arg_reseal_max_period = Args::select_value(
			cli_args.globals.sealing_mining.reseal_max_period,
			defaults.sealing_mining.reseal_max_period,
			fallback.sealing_mining.reseal_max_period,
		);
		self.arg_work_queue_size = Args::select_value(
			cli_args.globals.sealing_mining.work_queue_size,
			defaults.sealing_mining.work_queue_size,
			fallback.sealing_mining.work_queue_size,
		);
		self.arg_relay_set = Args::select_value(
			cli_args.globals.sealing_mining.relay_set,
			defaults.sealing_mining.relay_set,
			fallback.sealing_mining.relay_set,
		);
		self.arg_usd_per_tx = Args::select_value(
			cli_args.globals.sealing_mining.usd_per_tx,
			defaults.sealing_mining.usd_per_tx,
			fallback.sealing_mining.usd_per_tx,
		);
		self.arg_usd_per_eth = Args::select_value(
			cli_args.globals.sealing_mining.usd_per_eth,
			defaults.sealing_mining.usd_per_eth,
			fallback.sealing_mining.usd_per_eth,
		);
		self.arg_price_update_period = Args::select_value(
			cli_args.globals.sealing_mining.price_update_period,
			defaults.sealing_mining.price_update_period,
			fallback.sealing_mining.price_update_period,
		);
		self.arg_gas_floor_target = Args::select_value(
			cli_args.globals.sealing_mining.gas_floor_target,
			defaults.sealing_mining.gas_floor_target,
			fallback.sealing_mining.gas_floor_target,
		);
		self.arg_gas_cap = Args::select_value(
			cli_args.globals.sealing_mining.gas_cap,
			defaults.sealing_mining.gas_cap,
			fallback.sealing_mining.gas_cap,
		);
		self.arg_tx_queue_mem_limit = Args::select_value(
			cli_args.globals.sealing_mining.tx_queue_mem_limit,
			defaults.sealing_mining.tx_queue_mem_limit,
			fallback.sealing_mining.tx_queue_mem_limit,
		);
		self.arg_tx_queue_size = Args::select_value(
			cli_args.globals.sealing_mining.tx_queue_size,
			defaults.sealing_mining.tx_queue_size,
			fallback.sealing_mining.tx_queue_size,
		);
		self.arg_tx_queue_per_sender = Args::select_option(
			cli_args.globals.sealing_mining.tx_queue_per_sender,
			defaults.sealing_mining.tx_queue_per_sender,
			fallback.sealing_mining.tx_queue_per_sender,
		);
		self.arg_tx_queue_locals = Args::select_option(
			cli_args.globals.sealing_mining.tx_queue_locals,
			defaults.sealing_mining.tx_queue_locals,
			fallback.sealing_mining.tx_queue_locals,
		);
		self.arg_tx_queue_strategy = Args::select_value(
			cli_args.globals.sealing_mining.tx_queue_strategy,
			defaults.sealing_mining.tx_queue_strategy,
			fallback.sealing_mining.tx_queue_strategy,
		);
		self.arg_stratum_interface = Args::select_value(
			cli_args.globals.sealing_mining.stratum_interface,
			defaults.sealing_mining.stratum_interface,
			fallback.sealing_mining.stratum_interface,
		);
		self.arg_stratum_port = Args::select_value(
			cli_args.globals.sealing_mining.stratum_port,
			defaults.sealing_mining.stratum_port,
			fallback.sealing_mining.stratum_port,
		);
		self.arg_min_gas_price = Args::select_option(
			cli_args.globals.sealing_mining.min_gas_price,
			defaults.sealing_mining.min_gas_price,
			fallback.sealing_mining.min_gas_price,
		);
		self.arg_gas_price_percentile = Args::select_value(
			cli_args.globals.sealing_mining.gas_price_percentile,
			defaults.sealing_mining.gas_price_percentile,
			fallback.sealing_mining.gas_price_percentile,
		);
		self.arg_author = Args::select_option(
			cli_args.globals.sealing_mining.author,
			defaults.sealing_mining.author,
			fallback.sealing_mining.author,
		);
		self.arg_engine_signer = Args::select_option(
			cli_args.globals.sealing_mining.engine_signer,
			defaults.sealing_mining.engine_signer,
			fallback.sealing_mining.engine_signer,
		);
		self.arg_tx_gas_limit = Args::select_option(
			cli_args.globals.sealing_mining.tx_gas_limit,
			defaults.sealing_mining.tx_gas_limit,
			fallback.sealing_mining.tx_gas_limit,
		);
		self.arg_tx_time_limit = Args::select_option(
			cli_args.globals.sealing_mining.tx_time_limit,
			defaults.sealing_mining.tx_time_limit,
			fallback.sealing_mining.tx_time_limit,
		);
		self.arg_extra_data = Args::select_option(
			cli_args.globals.sealing_mining.extra_data,
			defaults.sealing_mining.extra_data,
			fallback.sealing_mining.extra_data,
		);
		self.arg_notify_work = Args::select_option(
			cli_args.globals.sealing_mining.notify_work,
			defaults.sealing_mining.notify_work,
			fallback.sealing_mining.notify_work,
		);
		self.arg_stratum_secret = Args::select_option(
			cli_args.globals.sealing_mining.stratum_secret,
			defaults.sealing_mining.stratum_secret,
			fallback.sealing_mining.stratum_secret,
		);
		self.arg_logging = Args::select_option(
			cli_args.globals.miscellaneous.logging,
			defaults.miscellaneous.logging,
			fallback.miscellaneous.logging,
		);
		self.arg_log_file = Args::select_option(
			cli_args.globals.miscellaneous.log_file,
			defaults.miscellaneous.log_file,
			fallback.miscellaneous.log_file,
		);
		self.arg_tracing = Args::select_value(
			cli_args.globals.footprint.tracing,
			defaults.footprint.tracing,
			fallback.footprint.tracing,
		);
		self.arg_pruning = Args::select_value(
			cli_args.globals.footprint.pruning,
			defaults.footprint.pruning,
			fallback.footprint.pruning,
		);
		self.arg_pruning_history = Args::select_value(
			cli_args.globals.footprint.pruning_history,
			defaults.footprint.pruning_history,
			fallback.footprint.pruning_history,
		);
		self.arg_pruning_memory = Args::select_value(
			cli_args.globals.footprint.pruning_memory,
			defaults.footprint.pruning_memory,
			fallback.footprint.pruning_memory,
		);
		self.arg_cache_size_db = Args::select_value(
			cli_args.globals.footprint.cache_size_db,
			defaults.footprint.cache_size_db,
			fallback.footprint.cache_size_db,
		);
		self.arg_cache_size_blocks = Args::select_value(
			cli_args.globals.footprint.cache_size_blocks,
			defaults.footprint.cache_size_blocks,
			fallback.footprint.cache_size_blocks,
		);
		self.arg_cache_size_queue = Args::select_value(
			cli_args.globals.footprint.cache_size_queue,
			defaults.footprint.cache_size_queue,
			fallback.footprint.cache_size_queue,
		);
		self.arg_cache_size_state = Args::select_value(
			cli_args.globals.footprint.cache_size_state,
			defaults.footprint.cache_size_state,
			fallback.footprint.cache_size_state,
		);
		self.arg_db_compaction = Args::select_value(
			cli_args.globals.footprint.db_compaction,
			defaults.footprint.db_compaction,
			fallback.footprint.db_compaction,
		);
		self.arg_fat_db = Args::select_value(
			cli_args.globals.footprint.fat_db,
			defaults.footprint.fat_db,
			fallback.footprint.fat_db,
		);
		self.arg_cache_size = Args::select_option(
			cli_args.globals.footprint.cache_size,
			defaults.footprint.cache_size,
			fallback.footprint.cache_size,
		);
		self.arg_num_verifiers = Args::select_option(
			cli_args.globals.footprint.num_verifiers,
			defaults.footprint.num_verifiers,
			fallback.footprint.num_verifiers,
		);
		self.arg_snapshot_threads = Args::select_option(
			cli_args.globals.snapshot.snapshot_threads,
			defaults.snapshot.snapshot_threads,
			fallback.snapshot.snapshot_threads,
		);
		self.arg_secretstore_http_interface = Args::select_value(
			cli_args.globals.secret_store.secretstore_http_interface,
			defaults.secret_store.secretstore_http_interface,
			fallback.secret_store.secretstore_http_interface,
		);
		self.arg_max_round_blocks_to_import = Args::select_value(
			cli_args.globals.sealing_mining.max_round_blocks_to_import,
			defaults.sealing_mining.max_round_blocks_to_import,
			fallback.sealing_mining.max_round_blocks_to_import,
		);
		self.arg_on_demand_request_backoff_start = Args::select_option(
			cli_args
				.globals
				.light_client
				.on_demand_request_backoff_start,
			defaults.light_client.on_demand_request_backoff_start,
			fallback.light_client.on_demand_request_backoff_start,
		);
		self.arg_on_demand_request_backoff_max = Args::select_option(
			cli_args.globals.light_client.on_demand_request_backoff_max,
			defaults.light_client.on_demand_request_backoff_max,
			fallback.light_client.on_demand_request_backoff_max,
		);
		self.arg_on_demand_request_backoff_rounds_max = Args::select_option(
			cli_args
				.globals
				.light_client
				.on_demand_request_backoff_rounds_max,
			defaults.light_client.on_demand_request_backoff_rounds_max,
			fallback.light_client.on_demand_request_backoff_rounds_max,
		);
		self.arg_on_demand_request_consecutive_failures = Args::select_option(
			cli_args
				.globals
				.light_client
				.on_demand_request_consecutive_failures,
			defaults.light_client.on_demand_request_consecutive_failures,
			fallback.light_client.on_demand_request_consecutive_failures,
		);
		self.arg_secretstore_srv_gen_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_srv_gen_contract,
			defaults.secret_store.secretstore_srv_gen_contract,
			fallback.secret_store.secretstore_srv_gen_contract,
		);
		self.arg_secretstore_srv_retr_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_srv_retr_contract,
			defaults.secret_store.secretstore_srv_retr_contract,
			fallback.secret_store.secretstore_srv_retr_contract,
		);
		self.arg_secretstore_doc_store_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_doc_store_contract,
			defaults.secret_store.secretstore_doc_store_contract,
			fallback.secret_store.secretstore_doc_store_contract,
		);
		self.arg_secretstore_doc_sretr_contract = Args::select_option(
			cli_args.globals.secret_store.secretstore_doc_sretr_contract,
			defaults.secret_store.secretstore_doc_sretr_contract,
			fallback.secret_store.secretstore_doc_sretr_contract,
		);
		self.arg_secretstore_nodes = Args::select_value(
			cli_args.globals.secret_store.secretstore_nodes,
			defaults.secret_store.secretstore_nodes,
			fallback.secret_store.secretstore_nodes,
		);
		self.arg_secretstore_server_set_contract = Args::select_option(
			cli_args
				.globals
				.secret_store
				.secretstore_server_set_contract,
			defaults.secret_store.secretstore_server_set_contract,
			fallback.secret_store.secretstore_server_set_contract,
		);
		self.arg_private_signer = Args::select_option(
			cli_args.globals.private_transactions.private_signer,
			defaults.private_transactions.private_signer,
			fallback.private_transactions.private_signer,
		);
	}

	fn absorb_global_flags(&mut self, cli_args: &ArgsInput, defaults: &Globals, fallback: &Globals) {
		self.arg_enable_signing_queue = cli_args.globals.account.enable_signing_queue
			|| defaults.account.enable_signing_queue
			|| fallback.account.enable_signing_queue;

		self.flag_fast_unlock = cli_args.globals.account.fast_unlock
			|| defaults.account.fast_unlock
			|| fallback.account.fast_unlock;

		self.flag_no_serve_light = cli_args.globals.networking.no_serve_light
			|| defaults.networking.no_serve_light
			|| fallback.networking.no_serve_light;

		self.flag_no_secretstore = cli_args.globals.secret_store.no_secretstore
			|| defaults.secret_store.no_secretstore
			|| fallback.secret_store.no_secretstore;

		self.flag_no_secretstore_http = cli_args.globals.secret_store.no_secretstore_http
			|| defaults.secret_store.no_secretstore_http
			|| fallback.secret_store.no_secretstore_http;

		self.flag_no_secretstore_auto_migrate =
			cli_args.globals.secret_store.no_secretstore_auto_migrate
				|| defaults.secret_store.no_secretstore_auto_migrate
				|| fallback.secret_store.no_secretstore_auto_migrate;

		self.flag_no_ws = cli_args.globals.websockets.no_ws
			|| defaults.websockets.no_ws
			|| fallback.websockets.no_ws;

		self.flag_no_ipc =
			cli_args.globals.ipc.no_ipc || defaults.ipc.no_ipc || fallback.ipc.no_ipc;

		self.flag_force_sealing = cli_args.globals.sealing_mining.force_sealing
			|| defaults.sealing_mining.force_sealing
			|| fallback.sealing_mining.force_sealing;

		self.flag_reseal_on_uncle = cli_args.globals.sealing_mining.reseal_on_uncle
			|| defaults.sealing_mining.reseal_on_uncle
			|| fallback.sealing_mining.reseal_on_uncle;

		self.flag_remove_solved = cli_args.globals.sealing_mining.remove_solved
			|| defaults.sealing_mining.remove_solved
			|| fallback.sealing_mining.remove_solved;

		self.flag_tx_queue_no_unfamiliar_locals = cli_args
			.globals
			.sealing_mining
			.tx_queue_no_unfamiliar_locals
			|| defaults.sealing_mining.tx_queue_no_unfamiliar_locals
			|| fallback.sealing_mining.tx_queue_no_unfamiliar_locals;

		self.flag_tx_queue_no_early_reject =
			cli_args.globals.sealing_mining.tx_queue_no_early_reject
				|| defaults.sealing_mining.tx_queue_no_early_reject
				|| fallback.sealing_mining.tx_queue_no_early_reject;

		self.flag_refuse_service_transactions =
			cli_args.globals.sealing_mining.refuse_service_transactions
				|| defaults.sealing_mining.refuse_service_transactions
				|| fallback.sealing_mining.refuse_service_transactions;

		self.flag_infinite_pending_block = cli_args.globals.sealing_mining.infinite_pending_block
			|| defaults.sealing_mining.infinite_pending_block
			|| fallback.sealing_mining.infinite_pending_block;

		self.flag_no_persistent_txqueue = cli_args.globals.sealing_mining.no_persistent_txqueue
			|| defaults.sealing_mining.no_persistent_txqueue
			|| fallback.sealing_mining.no_persistent_txqueue;

		self.flag_stratum = cli_args.globals.sealing_mining.stratum
			|| defaults.sealing_mining.stratum
			|| fallback.sealing_mining.stratum;

		self.flag_no_seal_check = cli_args.globals.import_export.no_seal_check
			|| defaults.import_export.no_seal_check
			|| fallback.import_export.no_seal_check;

		self.flag_can_restart = cli_args.globals.internal.can_restart
			|| defaults.internal.can_restart
			|| fallback.internal.can_restart;

		self.flag_no_color = cli_args.globals.miscellaneous.no_color
			|| defaults.miscellaneous.no_color
			|| fallback.miscellaneous.no_color;

		self.flag_no_config = cli_args.globals.miscellaneous.no_config
			|| defaults.miscellaneous.no_config
			|| fallback.miscellaneous.no_config;

		self.flag_scale_verifiers = cli_args.globals.footprint.scale_verifiers
			|| defaults.footprint.scale_verifiers
			|| fallback.footprint.scale_verifiers;

		self.flag_no_periodic_snapshot = cli_args.globals.snapshot.no_periodic_snapshot
			|| defaults.snapshot.no_periodic_snapshot
			|| fallback.snapshot.no_periodic_snapshot;

		self.flag_no_download = cli_args.globals.operating.no_download
			|| defaults.operating.no_download
			|| fallback.operating.no_download;

		self.flag_no_consensus = cli_args.globals.operating.no_consensus
			|| defaults.operating.no_consensus
			|| fallback.operating.no_consensus;

		self.flag_light = cli_args.globals.operating.light
			|| defaults.operating.light
			|| fallback.operating.light;

		self.flag_no_hardcoded_sync = cli_args.globals.operating.light
			|| defaults.operating.light
			|| fallback.operating.light;

		self.flag_force_direct = cli_args.globals.operating.force_direct
			|| defaults.operating.force_direct
			|| fallback.operating.force_direct;

		self.flag_unsafe_expose = cli_args.globals.convenience.unsafe_expose
			|| defaults.convenience.unsafe_expose
			|| fallback.convenience.unsafe_expose;

		self.flag_private_enabled = cli_args.globals.private_transactions.private_enabled
			|| defaults.private_transactions.private_enabled
			|| fallback.private_transactions.private_enabled;

		self.flag_private_state_offchain =
			cli_args.globals.private_transactions.private_state_offchain
				|| defaults.private_transactions.private_state_offchain
				|| fallback.private_transactions.private_state_offchain;

		self.flag_jsonrpc_allow_missing_blocks =
			cli_args.globals.http_json_rpc.jsonrpc_allow_missing_blocks
				|| defaults.http_json_rpc.jsonrpc_allow_missing_blocks
				|| fallback.http_json_rpc.jsonrpc_allow_missing_blocks;

		self.flag_no_jsonrpc = cli_args.globals.http_json_rpc.no_jsonrpc
			|| defaults.http_json_rpc.no_jsonrpc
			|| fallback.http_json_rpc.no_jsonrpc;

		self.flag_jsonrpc_no_keep_alive = cli_args.globals.http_json_rpc.jsonrpc_no_keep_alive
			|| defaults.http_json_rpc.jsonrpc_no_keep_alive
			|| fallback.http_json_rpc.jsonrpc_no_keep_alive;

		self.flag_jsonrpc_experimental = cli_args.globals.http_json_rpc.jsonrpc_experimental
			|| defaults.http_json_rpc.jsonrpc_experimental
			|| fallback.http_json_rpc.jsonrpc_experimental;

		self.flag_no_warp = cli_args.globals.networking.no_warp
			|| defaults.networking.no_warp
			|| fallback.networking.no_warp;

		self.flag_no_discovery = cli_args.globals.networking.no_discovery
			|| defaults.networking.no_discovery
			|| fallback.networking.no_discovery;

		self.flag_reserved_only = cli_args.globals.networking.reserved_only
			|| defaults.networking.reserved_only
			|| fallback.networking.reserved_only;

		self.flag_no_ancient_blocks = cli_args.globals.networking.no_ancient_blocks
			|| defaults.networking.no_ancient_blocks
			|| fallback.networking.no_ancient_blocks;

		self.flag_geth =
			cli_args.globals.legacy.geth || defaults.legacy.geth || fallback.legacy.geth;

		self.flag_import_geth_keys = cli_args.globals.legacy.import_geth_keys
			|| defaults.legacy.import_geth_keys
			|| fallback.legacy.import_geth_keys;
	}
}
