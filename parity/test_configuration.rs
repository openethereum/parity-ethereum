#[cfg(test)]
mod test_configuration{
	use cli::args::Args;
	use cli::parse_cli::ArgsInput;
	use cli::globals::Globals;
	use ethcore_logger::Config as LogConfig;
	use configuration::{Configuration, Cmd};
	use std::time::Duration;
	use params::{GasPricerConfig};
	use cache::CacheConfig;

	use std::io::Write;
	use std::fs::File;
	use std::str::FromStr;

	use tempfile::TempDir;
	use ethcore::miner::MinerOptions;
	use miner::pool::PrioritizationStrategy;
	use parity_rpc::NetworkSettings;
	use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};
	use types::ids::BlockId;
	use types::data_format::DataFormat;
	use account::{AccountCmd, NewAccount, ImportAccounts, ListAccounts};
	use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, ExportState};
	use dir::{Directories, default_hypervisor_path};
	use helpers::{default_network_config};
	use params::SpecType;
	use presale::ImportWallet;
	use rpc::WsConfiguration;
	use rpc_apis::ApiSet;
	use run::RunCmd;

	use network::{AllowIP, IpFilter};

	extern crate ipnetwork;
	use self::ipnetwork::IpNetwork;

	use super::*;


	fn intialize_with_out_of_the_box_defaults() -> (Args, ArgsInput, Globals, Globals) {
		let raw: ArgsInput = Default::default();
		let resolved: Args = Default::default();
		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_default.toml",
			"config_default.toml"
		).unwrap();

		(resolved, raw, user_defaults, fallback)
	}

	#[test]
	fn test_subcommand_account_new() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_account = true;
		conf.cmd_account_new = true;

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg: Cmd = Cmd::Account(
			AccountCmd::New(NewAccount{
				iterations: 10240,
				path: Directories::default().keys,
				password_file: None,
				spec: SpecType::default(),
			}));

		assert_eq!(conf, cmd_arg);

	}

	#[test]
	fn test_command_account_list() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();

		conf.cmd_account = true;
		conf.cmd_account_list = true;
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg: Cmd = Cmd::Account(
			AccountCmd::List(ListAccounts {
				path: Directories::default().keys,
				spec: SpecType::default(),
			}));

		assert_eq!(conf, cmd_arg);

	}

	#[test]
	fn test_command_account_import() {

		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_account = true;
		conf.cmd_account_import = true;
		conf.arg_account_import_path = Some(vec!["my_dir".into(), "another_dir".into()]);

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg: Cmd = Cmd::Account(AccountCmd::Import(ImportAccounts {
			from: vec!["my_dir".into(), "another_dir".into()],
			to: Directories::default().keys,
			spec: SpecType::default(),
		}));

		assert_eq!(conf, cmd_arg);
	}

	#[test]
	fn test_command_wallet_import() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_wallet = true;
		conf.cmd_wallet_import = true;
		conf.arg_wallet_import_path = Some("my_wallet.json".to_owned());
		conf.arg_password = vec!["pwd".into()];

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg = Cmd::ImportPresaleWallet(ImportWallet {
			iterations: 10240,
			path: Directories::default().keys,
			wallet_path: "my_wallet.json".into(),
			password_file: Some("pwd".into()),
			spec: SpecType::default(),
		});

		assert_eq!(conf, cmd_arg);
	}

	#[test]
	fn test_command_blockchain_import() {

		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_import = true;
		conf.arg_import_file = Some("blockchain.json".to_owned());

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg = Cmd::Blockchain(BlockchainCmd::Import(ImportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			format: Default::default(),
			pruning: Default::default(),
			pruning_history: 128,
			pruning_memory: 64,
			compaction: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			check_seal: true,
			with_color: !cfg!(windows),
			verifier_settings: Default::default(),
			light: false,
			max_round_blocks_to_import: 12,
		}));

		assert_eq!(conf, cmd_arg);
	}

	#[test]
	fn test_command_blockchain_export() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_export = true;
		conf.cmd_export_blocks = true;
		conf.arg_export_blocks_from = "1".to_owned();
		conf.arg_export_blocks_to = "latest".to_owned();
		conf.arg_export_blocks_file = Some("blockchain.json".to_owned());

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg = Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 128,
			pruning_memory: 64,
			format: Default::default(),
			compaction: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockId::Number(1),
			to_block: BlockId::Latest,
			check_seal: true,
			max_round_blocks_to_import: 12,
		}));
		assert_eq!(conf, cmd_arg);
	}
	#[test]
	fn test_command_state_export() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_export = true;
		conf.cmd_export_state = true;
		conf.arg_export_state_at = "latest".to_owned();
		conf.arg_export_state_min_balance = None;
		conf.arg_export_state_max_balance = None;
		conf.arg_export_state_file = Some("state.json".to_owned());

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg = Cmd::Blockchain(BlockchainCmd::ExportState(ExportState {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("state.json".into()),
			pruning: Default::default(),
			pruning_history: 128,
			pruning_memory: 64,
			format: Default::default(),
			compaction: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			at: BlockId::Latest,
			storage: true,
			code: true,
			min_balance: None,
			max_balance: None,
			max_round_blocks_to_import: 12,
		}));

		assert_eq!(conf, cmd_arg);
	}

	#[test]
	fn test_command_blockchain_export_with_custom_format() {

		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_export = true;
		conf.cmd_export_blocks = true;
		conf.arg_export_blocks_from = "1".to_owned();
		conf.arg_export_blocks_to = "latest".to_owned();
		conf.arg_export_blocks_format = Some("hex".to_owned());
		conf.arg_export_blocks_file = Some("blockchain.json".to_owned());

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let cmd_arg = Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 128,
			pruning_memory: 64,
			format: Some(DataFormat::Hex),
			compaction: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockId::Number(1),
			to_block: BlockId::Latest,
			check_seal: true,
			max_round_blocks_to_import: 12,
		}));
		assert_eq!(conf, cmd_arg);
	}
	#[test]
	fn test_command_signer_new_token() {

		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.cmd_signer = true;
		conf.cmd_signer_new_token = true;

		let conf = Configuration {
			args: conf,
		};

		let conf: Cmd = conf.into_command().unwrap().cmd;

		let expected = Directories::default().signer;
		let cmd_arg = Cmd::SignerToken(WsConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8546,
			apis: ApiSet::UnsafeContext,
			origins: Some(vec!["parity://*".into(),"chrome-extension://*".into(), "moz-extension://*".into()]),
			hosts: Some(vec![]),
			signer_path: expected.into(),
			support_token_api: false,
			max_connections: 100,
		}, LogConfig {
			color: !cfg!(windows),
			mode: None,
			file: None,
		} );

		assert_eq!(conf, cmd_arg);
	}

	#[test]
	fn test_ws_max_connections() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.arg_ws_max_connections = 1;

		let conf = Configuration {
			args: conf,
		};

		assert_eq!(conf.ws_config().unwrap(), WsConfiguration {
			max_connections: 1,
			..Default::default()
		});
	}

	#[test]
	fn test_run_cmd() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		let conf = conf.into_command().unwrap().cmd;

		let mut expected = RunCmd {
			allow_missing_blocks: false,
			cache_config: Default::default(),
			dirs: Default::default(),
			spec: Default::default(),
			pruning: Default::default(),
			pruning_history: 128,
			pruning_memory: 64,
			daemon: None,
			logger_config: Default::default(),
			miner_options: Default::default(),
			gas_price_percentile: 50,
			poll_lifetime: 60,
			ws_conf: Default::default(),
			http_conf: Default::default(),
			ipc_conf: Default::default(),
			net_conf: default_network_config(),
			network_id: None,
			warp_sync: true,
			warp_barrier: None,
			acc_conf: Default::default(),
			gas_pricer_conf: Default::default(),
			miner_extras: Default::default(),
			update_policy: UpdatePolicy {
				enable_downloading: true,
				require_consensus: true,
				filter: UpdateFilter::Critical,
				track: ReleaseTrack::Unknown,
				path: default_hypervisor_path(),
				max_size: 128 * 1024 * 1024,
				max_delay: 100,
				frequency: 20,
			},
			mode: Default::default(),
			tracing: Default::default(),
			compaction: Default::default(),
			geth_compatibility: false,
			experimental_rpcs: false,
			net_settings: Default::default(),
			secretstore_conf: Default::default(),
			private_provider_conf: Default::default(),
			private_encryptor_conf: Default::default(),
			private_tx_enabled: false,
			name: "".into(),
			custom_bootnodes: false,
			fat_db: Default::default(),
			snapshot_conf: Default::default(),
			stratum: None,
			check_seal: true,
			download_old_blocks: true,
			verifier_settings: Default::default(),
			serve_light: true,
			light: false,
			no_hardcoded_sync: false,
			no_persistent_txqueue: false,
			max_round_blocks_to_import: 12,
			on_demand_response_time_window: None,
			on_demand_request_backoff_start: None,
			on_demand_request_backoff_max: None,
			on_demand_request_backoff_rounds_max: None,
			on_demand_request_consecutive_failures: None,
		};
		expected.secretstore_conf.enabled = cfg!(feature = "secretstore");
		expected.secretstore_conf.http_enabled = cfg!(feature = "secretstore");

		assert_eq!(conf, Cmd::Run(expected));
	}

	#[test]
	fn should_parse_mining_options() {
		// given
		let mut mining_options = MinerOptions::default();

		// setting up 2 separate configs
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap(); // default config

		let mut conf2 = conf0.clone();
		conf2.arg_tx_queue_strategy = "gas_price".to_owned(); // modified config

		let conf0 = Configuration {
			args: conf0,
		};

		let conf2 = Configuration {
			args: conf2,
		};

		// then
		assert_eq!(conf0.miner_options().unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasPriceOnly;
		assert_eq!(conf2.miner_options().unwrap(), mining_options);
	}

	#[test]
	fn should_fail_on_force_reseal_and_reseal_min_period() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.arg_chain = "dev".to_owned();
		conf.flag_force_sealing = true;
		conf.arg_reseal_min_period = 0;

		let conf = Configuration {
			args: conf,
		};

		assert!(conf.miner_options().is_err());
	}

	#[test]
	fn should_parse_updater_options() {
		 let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		 conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		 conf0.arg_auto_update = "all".to_owned();
		 conf0.arg_auto_update_delay = 300;
		 conf0.flag_no_consensus = true;

		 let conf0 = Configuration {
			 args: conf0,
		 };

		assert_eq!(conf0.update_policy().unwrap(), UpdatePolicy {
			enable_downloading: true,
			require_consensus: false,
			filter: UpdateFilter::All,
			track: ReleaseTrack::Unknown,
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: 300,
			frequency: 20,
		});
	}

	#[test]
	fn should_parse_network_settings() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		 conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		 conf0.arg_identity = "testname".to_owned();
		 conf0.arg_chain = "goerli".to_owned();

		 let conf0 = Configuration {
			 args: conf0,
		 };

		// then
		assert_eq!(conf0.network_settings(), Ok(NetworkSettings {
			name: "testname".to_owned(),
			chain: "goerli".to_owned(),
			is_dev_chain: false,
			network_port: 30303,
			rpc_enabled: true,
			rpc_interface: "127.0.0.1".to_owned(),
			rpc_port: 8545,
		}));
	}

	#[test]
	fn should_parse_rpc_settings_with_geth_compatiblity() {
		// given
		fn assert(conf: Configuration) {
			let net = conf.network_settings().unwrap();
			assert_eq!(net.rpc_enabled, true);
			assert_eq!(net.rpc_interface, "0.0.0.0".to_owned());
			assert_eq!(net.rpc_port, 8000);
			assert_eq!(conf.rpc_cors(), None);
			assert_eq!(conf.rpc_apis(), "web3,eth".to_owned());
		}

		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		 conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		 let conf2 = conf0.clone();

		 conf0.arg_jsonrpc_port = 8000;
		 conf0.arg_jsonrpc_interface = "all".to_owned();
		 conf0.arg_jsonrpc_cors = "*".to_owned();
		 conf0.arg_jsonrpc_apis = "web3,eth".to_owned();

		 let conf0 = Configuration {
			 args: conf0,
		 };
		assert(conf0);
	}

	#[test]
	fn should_parse_rpc_hosts() {
		// given
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		 conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		 let mut conf1 = conf0.clone();
		 let mut conf2 = conf0.clone();
		 let mut conf3 = conf0.clone();

		conf1.arg_jsonrpc_hosts = "none".to_owned();
		conf2.arg_jsonrpc_hosts = "all".to_owned();
		conf3.arg_jsonrpc_hosts = "parity.io,something.io".to_owned();

		let conf0 = Configuration {
			args: conf0,
		};

		let conf1 = Configuration {
			args: conf1,
		};
		let conf2 = Configuration {
			args: conf2,
		};
		let conf3 = Configuration {
			args: conf3,
		};
		// then
		assert_eq!(conf0.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf1.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf2.rpc_hosts(), None);
		assert_eq!(conf3.rpc_hosts(), Some(vec!["parity.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_respect_only_min_peers_and_default() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.arg_min_peers = Some(5);

		let conf = Configuration{
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 5);
				assert_eq!(c.net_conf.max_peers, 50);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_respect_only_min_peers_and_greater_than_default() {
		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf.arg_min_peers = Some(500);

		let conf = Configuration{
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 500);
				assert_eq!(c.net_conf.max_peers, 500);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_parse_secretstore_cors() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		let mut conf1 = conf0.clone();
		let mut conf2 = conf0.clone();

		conf1.arg_secretstore_http_cors = "*".to_owned();
		conf2.arg_secretstore_http_cors = "http://parity.io,http://something.io".to_owned();

		let conf0 = Configuration{
			args: conf0,
		};

		let conf1 = Configuration{
			args: conf1,
		};

		let conf2 =  Configuration{
			args: conf2,
		};

		// then
		assert_eq!(conf0.secretstore_cors(), Some(vec![]));
		assert_eq!(conf1.secretstore_cors(), None);
		assert_eq!(conf2.secretstore_cors(), Some(vec!["http://parity.io".into(),"http://something.io".into()]));
	}

	#[test]
	fn ensures_sane_http_settings() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf0.arg_jsonrpc_server_threads = Some(0);
		conf0.arg_jsonrpc_max_payload = Some(0);

		let conf0 = Configuration {
			args: conf0,
		};

		// then things are adjusted to Just Work.
		let http_conf = conf0.http_config().unwrap();
		assert_eq!(http_conf.server_threads, 1);
		assert_eq!(http_conf.max_payload, 1);
	}

	#[test]
	fn jsonrpc_threading_defaults() {

		let (mut conf, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		assert_eq!(conf.arg_jsonrpc_server_threads, Some(4));
	}

	#[test]
	fn test_dev_preset() {

		let raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_dev.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_settings.chain, "dev");
				assert_eq!(c.gas_pricer_conf, GasPricerConfig::Fixed(0.into()));
				assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(0));
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_mining_preset() {

		let raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_mining.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 50);
				assert_eq!(c.net_conf.max_peers, 100);
				assert_eq!(c.ipc_conf.enabled, false);
				assert_eq!(c.miner_options.force_sealing, true);
				assert_eq!(c.miner_options.reseal_on_external_tx, true);
				assert_eq!(c.miner_options.reseal_on_own_tx, true);
				assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(4000));
				assert_eq!(c.miner_options.pool_limits.max_count, 8192);
				assert_eq!(c.cache_config, CacheConfig::new_with_total_cache_size(1024));
				assert_eq!(c.logger_config.mode.unwrap(), "miner=trace,own_tx=trace");
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_non_standard_ports_preset() {
		let raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_non_standard_ports.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_settings.network_port, 30305);
				assert_eq!(c.net_settings.rpc_port, 8645);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_insecure_preset() {

		let raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_insecure.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.update_policy.require_consensus, false);
				assert_eq!(c.net_settings.rpc_interface, "0.0.0.0");
				match c.http_conf.apis {
					ApiSet::List(set) => assert_eq!(set, ApiSet::All.list_apis()),
					_ => panic!("Incorrect rpc apis"),
				}
;
				assert_eq!(c.http_conf.hosts, None);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_dev_insecure_preset() {

		let raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_dev_insecure.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_settings.chain, "dev");
				assert_eq!(c.gas_pricer_conf, GasPricerConfig::Fixed(0.into()));
				assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(0));
				assert_eq!(c.update_policy.require_consensus, false);
				assert_eq!(c.net_settings.rpc_interface, "0.0.0.0");
				match c.http_conf.apis {
					ApiSet::List(set) => assert_eq!(set, ApiSet::All.list_apis()),
					_ => panic!("Incorrect rpc apis"),
				}
				// "web3,eth,net,personal,parity,parity_set,traces,rpc,parity_accounts");
				assert_eq!(c.http_conf.hosts, None);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_override_preset() {

		let mut raw: ArgsInput = Default::default();
		let mut conf: Args = Default::default();

		raw.globals.networking.min_peers = Some(99);

		let (user_defaults, fallback) = Args::generate_default_configuration(
			"config_mining.toml",
			"config_default.toml"
		).unwrap();

		conf.absorb_cli(raw, user_defaults, fallback).unwrap();

		let conf = Configuration {
			args: conf,
		};

		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 99);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_identity_arg() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		 conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf0.arg_identity = "Somebody".to_owned();

		let conf0 = Configuration {
			args: conf0
		};

		match conf0.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.name, "Somebody");
				assert!(c.net_conf.client_version.starts_with("OpenEthereum/Somebody/"));
			}
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_apply_ports_shift() {
		// give
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		let mut conf1 = conf0.clone();

		conf0.arg_ports_shift = 1;
		conf0.flag_stratum = true;

		conf1.arg_ports_shift = 1;
		conf1.arg_jsonrpc_port = 8544;

		let conf0 = Configuration {
			args: conf0
		};

		let conf1 = Configuration {
			args: conf1
		};

		assert_eq!(conf0.net_addresses().unwrap().0.port(), 30304);
		assert_eq!(conf0.network_settings().unwrap().network_port, 30304);
		assert_eq!(conf0.network_settings().unwrap().rpc_port, 8546);
		assert_eq!(conf0.http_config().unwrap().port, 8546);
		assert_eq!(conf0.ws_config().unwrap().port, 8547);
		assert_eq!(conf0.secretstore_config().unwrap().port, 8084);
		assert_eq!(conf0.secretstore_config().unwrap().http_port, 8083);
		assert_eq!(conf0.stratum_options().unwrap().unwrap().port, 8009);

		assert_eq!(conf1.net_addresses().unwrap().0.port(), 30304);
		assert_eq!(conf1.network_settings().unwrap().network_port, 30304);
		assert_eq!(conf1.network_settings().unwrap().rpc_port, 8545);
		assert_eq!(conf1.http_config().unwrap().port, 8545);
		assert_eq!(conf1.ws_config().unwrap().port, 8547);
		assert_eq!(conf1.secretstore_config().unwrap().port, 8084);
		assert_eq!(conf1.secretstore_config().unwrap().http_port, 8083);
	}

	#[test]
	fn should_resolve_external_nat_hosts() {

		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		let mut conf1 = conf0.clone();
		let mut conf2 = conf0.clone();
		let mut conf3 = conf0.clone();
		let mut conf4 = conf0.clone();

		conf0.arg_nat = "extip:1.1.1.1".to_owned();

		let conf0 = Configuration {
			args: conf0
		};

		conf1.arg_nat = "extip:192.168.1.1:123".to_owned();
		let conf1 = Configuration {
			args: conf1
		};

		conf2.arg_nat = "extip:ethereum.org".to_owned();
		let conf2 = Configuration {
			args: conf2
		};

		conf3.arg_nat = "extip:ethereum.org:whatever bla bla 123".to_owned();
		let conf3 = Configuration {
			args: conf3
		};

		conf4.arg_nat = "extip:blabla".to_owned();
		let conf4 = Configuration {
			args: conf4
		};
		// Ip works
		assert_eq!(conf0.net_addresses().unwrap().1.unwrap().ip().to_string(), "1.1.1.1");
		assert_eq!(conf0.net_addresses().unwrap().1.unwrap().port(), 30303);

		// Ip with port works, port is discarded
		assert_eq!(conf1.net_addresses().unwrap().1.unwrap().ip().to_string(), "192.168.1.1");
		assert_eq!(conf1.net_addresses().unwrap().1.unwrap().port(), 30303);

		// Hostname works
		assert!(conf2.net_addresses().unwrap().1.is_some());
		assert_eq!(conf2.net_addresses().unwrap().1.unwrap().port(), 30303);

		// Hostname works, garbage at the end is discarded
		assert!(conf3.net_addresses().unwrap().1.is_some());
		assert_eq!(conf3.net_addresses().unwrap().1.unwrap().port(), 30303);

		// Garbage is error
		assert!(conf4.net_addresses().is_err());
	}

	#[test]
	fn should_expose_all_servers() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf0.flag_unsafe_expose = true;

		let conf0 = Configuration {
			args: conf0
		};

		assert_eq!(&conf0.network_settings().unwrap().rpc_interface, "0.0.0.0");
		assert_eq!(&conf0.http_config().unwrap().interface, "0.0.0.0");
		assert_eq!(conf0.http_config().unwrap().hosts, None);
		assert_eq!(&conf0.ws_config().unwrap().interface, "0.0.0.0");
		assert_eq!(conf0.ws_config().unwrap().hosts, None);
		assert_eq!(conf0.ws_config().unwrap().origins, None);
		assert_eq!(&conf0.secretstore_config().unwrap().interface, "0.0.0.0");
		assert_eq!(&conf0.secretstore_config().unwrap().http_interface, "0.0.0.0");
	}

	#[test]
	fn allow_ips() {

		let (mut all, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		all.absorb_cli(raw, user_defaults, fallback).unwrap();

		let mut private = all.clone();
		let mut block_custom = all.clone();
		let mut combo = all.clone();
		let mut ipv6_custom_public = all.clone();
		let mut ipv6_custom_private = all.clone();

		all.arg_allow_ips = "all".to_owned();
		let all = Configuration {
			args: all
		};

		private.arg_allow_ips = "private".to_owned();
		let private = Configuration {
			args: private
		};

		block_custom.arg_allow_ips = "-10.0.0.0/8".to_owned();
		let block_custom = Configuration {
			args: block_custom
		};

		combo.arg_allow_ips = "public 10.0.0.0/8 -1.0.0.0/8".to_owned();
		let combo = Configuration {
			args: combo
		};

		ipv6_custom_public.arg_allow_ips = "public fc00::/7".to_owned();
		let ipv6_custom_public = Configuration {
			args: ipv6_custom_public
		};

		ipv6_custom_private.arg_allow_ips = "private -fc00::/7".to_owned();
		let ipv6_custom_private = Configuration {
			args: ipv6_custom_private
		};

		assert_eq!(all.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::All,
			custom_allow: vec![],
			custom_block: vec![],
		});

		assert_eq!(private.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::Private,
			custom_allow: vec![],
			custom_block: vec![],
		});

		assert_eq!(block_custom.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::All,
			custom_allow: vec![],
			custom_block: vec![IpNetwork::from_str("10.0.0.0/8").unwrap()],
		});

		assert_eq!(combo.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::Public,
			custom_allow: vec![IpNetwork::from_str("10.0.0.0/8").unwrap()],
			custom_block: vec![IpNetwork::from_str("1.0.0.0/8").unwrap()],
		});

		assert_eq!(ipv6_custom_public.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::Public,
			custom_allow: vec![IpNetwork::from_str("fc00::/7").unwrap()],
			custom_block: vec![],
		});

		assert_eq!(ipv6_custom_private.ip_filter().unwrap(), IpFilter {
			predefined: AllowIP::Private,
			custom_allow: vec![],
			custom_block: vec![IpNetwork::from_str("fc00::/7").unwrap()],
		});
	}

	#[test]
	fn should_use_correct_cache_path_if_base_is_set() {
		use std::path;

		let (mut std, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		std.absorb_cli(raw, user_defaults, fallback).unwrap();

		let mut base = std.clone();
		base.arg_base_path = Some("/test".to_owned());

		let std = Configuration {
			args: std
		};

		let base = Configuration {
			args: base
		};

		let base_path = ::dir::default_data_path();
		let local_path = ::dir::default_local_path();
		assert_eq!(std.directories().cache, dir::helpers::replace_home_and_local(&base_path, &local_path, ::dir::CACHE_PATH));
		assert_eq!(path::Path::new(&base.directories().cache), path::Path::new("/test/cache"));
	}

	#[test]
	fn should_respect_only_max_peers_and_default() {

		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

		conf0.arg_max_peers = Some(50);

	   let conf0 = Configuration {
		   args: conf0
	   };

		match conf0.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 25);
				assert_eq!(c.net_conf.max_peers, 50);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_respect_only_max_peers_less_than_default() {
		let (mut conf0, raw, user_defaults, fallback) = intialize_with_out_of_the_box_defaults();
		conf0.absorb_cli(raw, user_defaults, fallback).unwrap();

	   conf0.arg_max_peers = Some(5);

	   let conf0 = Configuration {
		   args: conf0
	   };

		match conf0.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 5);
				assert_eq!(c.net_conf.max_peers, 5);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

}
