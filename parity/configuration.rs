// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::time::Duration;
use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::collections::{HashSet, BTreeMap};
use std::iter::FromIterator;
use std::cmp;
use cli::args::{Args, ArgsError};
use hash::keccak;
use ethereum_types::{U256, H256, Address};
use parity_version::{version_data, version};
use bytes::Bytes;
use ansi_term::Colour;
use sync::{NetworkConfiguration, validate_node_url, self};
use parity_crypto::publickey::{Secret, Public};
use ethcore::miner::{stratum, MinerOptions};
use snapshot::SnapshotConfiguration;
use miner::pool;
use verification::queue::VerifierSettings;

use rpc::{IpcConfiguration, HttpConfiguration, WsConfiguration};
use parity_rpc::NetworkSettings;
use cache::CacheConfig;
use helpers::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_price, parity_ipc_path, to_bootnodes, to_addresses, to_address, to_queue_strategy, to_queue_penalization};
use dir::helpers::{replace_home, replace_home_and_local};
use params::{ResealPolicy, AccountsConfig, GasPricerConfig, MinerExtras, SpecType};
use ethcore_logger::Config as LogConfig;
use dir::{self, Directories, default_hypervisor_path, default_local_path, default_data_path};
use ethcore_private_tx::{ProviderConfig, EncryptorConfig};
use secretstore::{NodeSecretKey, Configuration as SecretStoreConfiguration, ContractAddress as SecretStoreContractAddress};
use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};
use run::RunCmd;
use types::data_format::DataFormat;
use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, KillBlockchain, ExportState, ResetBlockchain};
use export_hardcoded_sync::ExportHsyncCmd;
use presale::ImportWallet;
use account::{AccountCmd, NewAccount, ListAccounts, ImportAccounts};
use snapshot_cmd::{self, SnapshotCommand};
use network::{IpFilter, NatType};

const DEFAULT_MAX_PEERS: u16 = 50;
const DEFAULT_MIN_PEERS: u16 = 25;
pub const ETHERSCAN_ETH_PRICE_ENDPOINT: &str = "https://api.etherscan.io/api?module=stats&action=ethprice";

#[derive(Debug, PartialEq)]
pub enum Cmd {
	Run(RunCmd),
	// Version, // NOTE: this is automatically handled by structopt
	Account(AccountCmd),
	ImportPresaleWallet(ImportWallet),
	Blockchain(BlockchainCmd),
	SignerToken(WsConfiguration, LogConfig),
	SignerSign {
		id: Option<usize>,
		pwfile: Option<PathBuf>,
		port: u16,
		authfile: PathBuf,
	},
	SignerList {
		port: u16,
		authfile: PathBuf
	},
	SignerReject {
		id: Option<usize>,
		port: u16,
		authfile: PathBuf
	},
	Snapshot(SnapshotCommand),
	Hash(Option<String>),
	ExportHardcodedSync(ExportHsyncCmd),
}

#[derive(Debug, PartialEq)]
pub struct Execute {
	pub logger: LogConfig,
	pub cmd: Cmd,
}

/// Configuration for the Parity client.
#[derive(Debug, PartialEq)]
pub struct Configuration {
	/// Arguments to be interpreted.
	pub args: Args,
}

impl Configuration {
	/// Parses a configuration from a list of command line arguments.
	///
	/// # Example
	///
	/// ```
	/// let _cfg = openethereum::Configuration::parse_cli(&["--light", "--chain", "kovan"]).unwrap();
	/// ```
	pub fn parse_cli<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
		let config = Configuration {
			args: Args::parse()?,
		};

		Ok(config)
	}

	pub(crate) fn into_command(self) -> Result<Execute, String> {
		let dirs = self.directories();
		let pruning = self.args.arg_pruning.parse()?;
		let pruning_history = self.args.arg_pruning_history;
		let spec = self.chain()?;
		let mode = match self.args.arg_mode.as_ref() {
			"last" => None,
			mode => Some(to_mode(&mode, self.args.arg_mode_timeout, self.args.arg_mode_alarm)?),
		};
		let update_policy = self.update_policy()?;
		let logger_config = self.logger_config();
		let ws_conf = self.ws_config()?;
		let snapshot_conf = self.snapshot_config()?;
		let http_conf = self.http_config()?;
		let ipc_conf = self.ipc_config()?;
		let net_conf = self.net_config()?;
		let network_id = self.network_id();
		let cache_config = self.cache_config();
		let tracing = self.args.arg_tracing.parse()?;
		let fat_db = self.args.arg_fat_db.parse()?;
		let compaction = self.args.arg_db_compaction.parse()?;
		let warp_sync = !self.args.flag_no_warp;
		let geth_compatibility = self.args.flag_geth;
		let experimental_rpcs = self.args.flag_jsonrpc_experimental;
		let secretstore_conf = self.secretstore_config()?;
		let format = self.format()?;

		let key_iterations = self.args.arg_keys_iterations;
		if key_iterations == 0 {
			return Err("--key-iterations must be non-zero".into());
		}

		let cmd = if self.args.cmd_signer {
			let authfile = ::signer::codes_path(&ws_conf.signer_path);

			if self.args.cmd_signer_new_token {
				Cmd::SignerToken(ws_conf, logger_config.clone())
			} else if self.args.cmd_signer_sign {
				let pwfile = self.accounts_config()?.password_files.first().map(|pwfile| {
					PathBuf::from(pwfile)
				});
				Cmd::SignerSign {
					id: self.args.arg_signer_sign_id,
					pwfile: pwfile,
					port: ws_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_signer_reject {
				Cmd::SignerReject {
					id: self.args.arg_signer_reject_id,
					port: ws_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_signer_list {
				Cmd::SignerList {
					port: ws_conf.port,
					authfile: authfile,
				}
			} else {
				unreachable!();
			}
		} else if self.args.cmd_tools && self.args.cmd_tools_hash {
			Cmd::Hash(self.args.arg_tools_hash_file)
		} else if self.args.cmd_db && self.args.cmd_db_reset {
			Cmd::Blockchain(BlockchainCmd::Reset(ResetBlockchain {
				dirs,
				spec,
				pruning,
				pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				tracing,
				fat_db,
				compaction,
				cache_config,
				num: self.args.arg_db_reset_num,
			}))
		} else if self.args.cmd_db && self.args.cmd_db_kill {
			Cmd::Blockchain(BlockchainCmd::Kill(KillBlockchain {
				spec: spec,
				dirs: dirs,
				pruning: pruning,
			}))
		} else if self.args.cmd_account {
			let account_cmd = if self.args.cmd_account_new {
				let new_acc = NewAccount {
					iterations: key_iterations,
					path: dirs.keys,
					spec: spec,
					password_file: self.accounts_config()?.password_files.first().map(|x| x.to_owned()),
				};
				AccountCmd::New(new_acc)
			} else if self.args.cmd_account_list {
				let list_acc = ListAccounts {
					path: dirs.keys,
					spec: spec,
				};
				AccountCmd::List(list_acc)
			} else if self.args.cmd_account_import {
				let import_acc = ImportAccounts {
					from: self.args.arg_account_import_path.expect("CLI argument is required; qed").clone(),
					to: dirs.keys,
					spec: spec,
				};
				AccountCmd::Import(import_acc)
			} else {
				unreachable!();
			};
			Cmd::Account(account_cmd)
		} else if self.args.cmd_wallet {
			let presale_cmd = ImportWallet {
				iterations: key_iterations,
				path: dirs.keys,
				spec: spec,
				wallet_path: self.args.arg_wallet_import_path.clone().unwrap(),
				password_file: self.accounts_config()?.password_files.first().map(|x| x.to_owned()),
			};
			Cmd::ImportPresaleWallet(presale_cmd)
		} else if self.args.cmd_import {
			let import_cmd = ImportBlockchain {
				spec: spec,
				cache_config: cache_config,
				dirs: dirs,
				file_path: self.args.arg_import_file.clone(),
				format: format,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				compaction: compaction,
				tracing: tracing,
				fat_db: fat_db,
				check_seal: !self.args.flag_no_seal_check,
				with_color: logger_config.color,
				verifier_settings: self.verifier_settings(),
				light: self.args.flag_light,
				max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
			};
			Cmd::Blockchain(BlockchainCmd::Import(import_cmd))
		} else if self.args.cmd_export {
			if self.args.cmd_export_blocks {
				let export_cmd = ExportBlockchain {
					spec: spec,
					cache_config: cache_config,
					dirs: dirs,
					file_path: self.args.arg_export_blocks_file.clone(),
					format: format,
					pruning: pruning,
					pruning_history: pruning_history,
					pruning_memory: self.args.arg_pruning_memory,
					compaction: compaction,
					tracing: tracing,
					fat_db: fat_db,
					from_block: to_block_id(&self.args.arg_export_blocks_from)?,
					to_block: to_block_id(&self.args.arg_export_blocks_to)?,
					check_seal: !self.args.flag_no_seal_check,
					max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
				};
				Cmd::Blockchain(BlockchainCmd::Export(export_cmd))
			} else if self.args.cmd_export_state {
				let export_cmd = ExportState {
					spec: spec,
					cache_config: cache_config,
					dirs: dirs,
					file_path: self.args.arg_export_state_file.clone(),
					format: format,
					pruning: pruning,
					pruning_history: pruning_history,
					pruning_memory: self.args.arg_pruning_memory,
					compaction: compaction,
					tracing: tracing,
					fat_db: fat_db,
					at: to_block_id(&self.args.arg_export_state_at)?,
					storage: !self.args.flag_export_state_no_storage,
					code: !self.args.flag_export_state_no_code,
					min_balance: self.args.arg_export_state_min_balance.and_then(|s| to_u256(&s).ok()),
					max_balance: self.args.arg_export_state_max_balance.and_then(|s| to_u256(&s).ok()),
					max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
				};
				Cmd::Blockchain(BlockchainCmd::ExportState(export_cmd))
			} else {
				unreachable!();
			}
		} else if self.args.cmd_snapshot {
			let snapshot_cmd = SnapshotCommand {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_snapshot_file.clone(),
				kind: snapshot_cmd::Kind::Take,
				block_at: to_block_id(&self.args.arg_snapshot_at)?,
				max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
				snapshot_conf: snapshot_conf,
			};
			Cmd::Snapshot(snapshot_cmd)
		} else if self.args.cmd_restore {
			let restore_cmd = SnapshotCommand {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_restore_file.clone(),
				kind: snapshot_cmd::Kind::Restore,
				block_at: to_block_id("latest")?, // unimportant.
				max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
				snapshot_conf: snapshot_conf,
			};
			Cmd::Snapshot(restore_cmd)
		} else if self.args.cmd_export_hardcoded_sync {
			let export_hs_cmd = ExportHsyncCmd {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				compaction: compaction,
			};
			Cmd::ExportHardcodedSync(export_hs_cmd)
		} else {
			let daemon = if self.args.cmd_daemon {
				Some(self.args.arg_daemon_pid_file.clone().expect("CLI argument is required; qed"))
			} else {
				None
			};

			let verifier_settings = self.verifier_settings();
			let (private_provider_conf, private_enc_conf, private_tx_enabled) = self.private_provider_config()?;

			let run_cmd = RunCmd {
				cache_config,
				dirs,
				spec,
				pruning,
				pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				daemon,
				logger_config: logger_config.clone(),
				miner_options: self.miner_options()?,
				gas_price_percentile: self.args.arg_gas_price_percentile,
				poll_lifetime: self.args.arg_poll_lifetime,
				ws_conf,
				snapshot_conf,
				http_conf,
				ipc_conf,
				net_conf,
				network_id,
				acc_conf: self.accounts_config()?,
				gas_pricer_conf: self.gas_pricer_config()?,
				miner_extras: self.miner_extras()?,
				stratum: self.stratum_options()?,
				update_policy,
				allow_missing_blocks: self.args.flag_jsonrpc_allow_missing_blocks,
				mode,
				tracing,
				fat_db,
				compaction,
				warp_sync,
				warp_barrier: self.args.arg_warp_barrier,
				geth_compatibility,
				experimental_rpcs,
				net_settings: self.network_settings()?,
				secretstore_conf,
				private_provider_conf,
				private_encryptor_conf: private_enc_conf,
				private_tx_enabled,
				name: self.args.arg_identity,
				custom_bootnodes: self.args.arg_bootnodes.is_some(),
				check_seal: !self.args.flag_no_seal_check,
				download_old_blocks: !self.args.flag_no_ancient_blocks,
				verifier_settings,
				serve_light: !self.args.flag_no_serve_light,
				light: self.args.flag_light,
				no_persistent_txqueue: self.args.flag_no_persistent_txqueue,
				no_hardcoded_sync: self.args.flag_no_hardcoded_sync,
				max_round_blocks_to_import: self.args.arg_max_round_blocks_to_import,
				on_demand_response_time_window: self.args.arg_on_demand_response_time_window,
				on_demand_request_backoff_start: self.args.arg_on_demand_request_backoff_start,
				on_demand_request_backoff_max: self.args.arg_on_demand_request_backoff_max,
				on_demand_request_backoff_rounds_max: self.args.arg_on_demand_request_backoff_rounds_max,
				on_demand_request_consecutive_failures: self.args.arg_on_demand_request_consecutive_failures,
				sync_until: self.args.arg_sync_until,
			};
			Cmd::Run(run_cmd)
		};

		Ok(Execute {
			logger: logger_config,
			cmd,
		})
	}

	fn miner_extras(&self) -> Result<MinerExtras, String> {
		let floor = to_u256(&self.args.arg_gas_floor_target)?;
		let ceil = to_u256(&self.args.arg_gas_cap)?;
		let extras = MinerExtras {
			author: self.author()?,
			extra_data: self.extra_data()?,
			gas_range_target: (floor, ceil),
			engine_signer: self.engine_signer()?,
			work_notify: self.work_notify(),
			local_accounts: HashSet::from_iter(to_addresses(&self.args.arg_tx_queue_locals)?.into_iter()),
		};

		Ok(extras)
	}

	fn author(&self) -> Result<Address, String> {
		to_address(self.args.arg_author.clone())
	}

	fn engine_signer(&self) -> Result<Address, String> {
		to_address(self.args.arg_engine_signer.clone())
	}

	fn format(&self) -> Result<Option<DataFormat>, String> {
		match self.args.arg_import_format.clone()
				.or(self.args.arg_export_blocks_format.clone())
				.or(self.args.arg_export_state_format.clone()) {
			Some(ref f) => Ok(Some(f.parse()?)),
			None => Ok(None),
		}
	}

	fn cache_config(&self) -> CacheConfig {
		match self.args.arg_cache_size{
			Some(size) => CacheConfig::new_with_total_cache_size(size),
			None => CacheConfig::new(
				self.args.arg_cache_size_db,
				self.args.arg_cache_size_blocks,
				self.args.arg_cache_size_queue,
				self.args.arg_cache_size_state,
			),
		}
	}

	/// returns logger config
	pub fn logger_config(&self) -> LogConfig {
		LogConfig {
			mode: self.args.arg_logging.clone(),
			color: !self.args.flag_no_color && !cfg!(windows),
			file: self.args.arg_log_file.as_ref().map(|log_file| replace_home(&self.directories().base, log_file)),
		}
	}

	pub fn chain(&self) -> Result<SpecType, String> {
		let name = self.args.arg_chain.clone();
		Ok(name.parse()?)
	}

	pub fn is_dev_chain(&self) -> Result<bool, String> {
		Ok(self.chain()? == SpecType::Dev)
	}

	pub fn max_peers(&self) -> u32 {
		self.args.arg_max_peers
			.or(cmp::max(self.args.arg_min_peers, Some(DEFAULT_MAX_PEERS)))
			.unwrap_or(DEFAULT_MAX_PEERS) as u32
	}

	pub fn ip_filter(&self) -> Result<IpFilter, String> {
		match IpFilter::parse(self.args.arg_allow_ips.as_str()) {
			Ok(allow_ip) => Ok(allow_ip),
			Err(_) => Err("Invalid IP filter value".to_owned()),
		}
	}

	pub fn min_peers(&self) -> u32 {
		self.args.arg_min_peers
			.or(cmp::min(self.args.arg_max_peers, Some(DEFAULT_MIN_PEERS)))
			.unwrap_or(DEFAULT_MIN_PEERS) as u32
	}

	pub fn max_pending_peers(&self) -> u32 {
		self.args.arg_max_pending_peers as u32
	}

	pub fn snapshot_peers(&self) -> u32 {
		self.args.arg_snapshot_peers as u32
	}

	pub fn work_notify(&self) -> Vec<String> {
		self.args.arg_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	pub fn accounts_config(&self) -> Result<AccountsConfig, String> {
		let cfg = AccountsConfig {
			iterations: self.args.arg_keys_iterations,
			refresh_time: self.args.arg_accounts_refresh,
		testnet: false, // FIXME: legacy option, should ideally be deleted
			password_files: self.args.arg_password.iter().map(|s| replace_home(&self.directories().base, s)).collect(),
			unlocked_accounts: to_addresses(&self.args.arg_unlock)?,
			enable_fast_unlock: self.args.flag_fast_unlock,
		};

		Ok(cfg)
	}

	pub fn stratum_options(&self) -> Result<Option<stratum::Options>, String> {
	if self.args.flag_stratum {
			Ok(Some(stratum::Options {
				io_path: self.directories().db,
				listen_addr: self.stratum_interface(),
			port: self.args.arg_ports_shift + self.args.arg_stratum_port,
				secret: self.args.arg_stratum_secret.as_ref().map(|s| s.parse::<H256>().unwrap_or_else(|_| keccak(s))),
			}))
		} else { Ok(None) }
	}


	pub fn miner_options(&self) -> Result<MinerOptions, String> {
		let is_dev_chain = self.is_dev_chain()?;
		if is_dev_chain && self.args.flag_force_sealing && self.args.arg_reseal_min_period == 0 {
			return Err("Force sealing can't be used with reseal_min_period = 0".into());
		}

		let reseal = self.args.arg_reseal_on_txs.parse::<ResealPolicy>()?;

		let options = MinerOptions {
			force_sealing: self.args.flag_force_sealing,
			reseal_on_external_tx: reseal.external,
			reseal_on_own_tx: reseal.own,
			reseal_on_uncle: self.args.flag_reseal_on_uncle,
			reseal_min_period: Duration::from_millis(self.args.arg_reseal_min_period),
			reseal_max_period: Duration::from_millis(self.args.arg_reseal_max_period),

			pending_set: to_pending_set(&self.args.arg_relay_set)?,
			work_queue_size: self.args.arg_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
			infinite_pending_block: self.args.flag_infinite_pending_block,

			tx_queue_penalization: to_queue_penalization(self.args.arg_tx_time_limit)?,
			tx_queue_strategy: to_queue_strategy(&self.args.arg_tx_queue_strategy)?,
			tx_queue_no_unfamiliar_locals: self.args.flag_tx_queue_no_unfamiliar_locals,
			refuse_service_transactions: self.args.flag_refuse_service_transactions,

			pool_limits: self.pool_limits()?,
			pool_verification_options: self.pool_verification_options()?,
		};

		Ok(options)
	}

	pub fn pool_limits(&self) -> Result<pool::Options, String> {
		let max_count = self.args.arg_tx_queue_size;

		Ok(pool::Options {
			max_count,
			max_per_sender: self.args.arg_tx_queue_per_sender.unwrap_or_else(|| cmp::max(16, max_count / 100)),
			max_mem_usage: if self.args.arg_tx_queue_mem_limit > 0 {
				self.args.arg_tx_queue_mem_limit as usize * 1024 * 1024
			} else {
				usize::max_value()
			},
		})
	}

	pub fn pool_verification_options(&self) -> Result<pool::verifier::Options, String>{
		Ok(pool::verifier::Options {
			// NOTE min_gas_price and block_gas_limit will be overwritten right after start.
			minimal_gas_price: U256::from(20_000_000) * 1_000u32,
			block_gas_limit: U256::max_value(),
			tx_gas_limit: match self.args.arg_tx_gas_limit {
				Some(ref d) => to_u256(d)?,
				None => U256::max_value(),
			},
			no_early_reject: self.args.flag_tx_queue_no_early_reject,
		})
	}

	pub fn secretstore_config(&self) -> Result<SecretStoreConfiguration, String> {
		Ok(SecretStoreConfiguration {
			enabled: self.secretstore_enabled(),
			http_enabled: self.secretstore_http_enabled(),
			auto_migrate_enabled: self.secretstore_auto_migrate_enabled(),
			acl_check_contract_address: self.secretstore_acl_check_contract_address()?,
			service_contract_address: self.secretstore_service_contract_address()?,
			service_contract_srv_gen_address: self.secretstore_service_contract_srv_gen_address()?,
			service_contract_srv_retr_address: self.secretstore_service_contract_srv_retr_address()?,
			service_contract_doc_store_address: self.secretstore_service_contract_doc_store_address()?,
			service_contract_doc_sretr_address: self.secretstore_service_contract_doc_sretr_address()?,
			self_secret: self.secretstore_self_secret()?,
			nodes: self.secretstore_nodes()?,
			key_server_set_contract_address: self.secretstore_key_server_set_contract_address()?,
			interface: self.secretstore_interface(),
			port: self.args.arg_ports_shift + self.args.arg_secretstore_port,
			http_interface: self.secretstore_http_interface(),
			http_port: self.args.arg_ports_shift + self.args.arg_secretstore_http_port,
			data_path: self.directories().secretstore,
			admin_public: self.secretstore_admin_public()?,
			cors: self.secretstore_cors()
		})
	}

	pub fn gas_pricer_config(&self) -> Result<GasPricerConfig, String> {
		fn wei_per_gas(usd_per_tx: f32, usd_per_eth: f32) -> U256 {
			let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
			let gas_per_tx: f32 = 21000.0;
			let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
			U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()
		}

		if let Some(dec) = self.args.arg_min_gas_price {
			return Ok(GasPricerConfig::Fixed(U256::from(dec)));
		} else if self.chain()? != SpecType::Foundation {
			return Ok(GasPricerConfig::Fixed(U256::zero()));
		}

		let usd_per_tx = to_price(&self.args.arg_usd_per_tx)?;

		if "auto" == self.args.arg_usd_per_eth {
			Ok(GasPricerConfig::Calibrated {
				usd_per_tx: usd_per_tx,
				recalibration_period: to_duration(self.args.arg_price_update_period.as_str())?,
				api_endpoint: ETHERSCAN_ETH_PRICE_ENDPOINT.to_string(),
			})
		} else if let Ok(usd_per_eth_parsed) = to_price(&self.args.arg_usd_per_eth) {
			let wei_per_gas = wei_per_gas(usd_per_tx, usd_per_eth_parsed);

			info!(
				"Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)",
				Colour::White.bold().paint(format!("US${:.2}", usd_per_eth_parsed)),
				Colour::Yellow.bold().paint(format!("{}", wei_per_gas))
			);

			Ok(GasPricerConfig::Fixed(wei_per_gas))
		} else {
			Ok(GasPricerConfig::Calibrated {
				usd_per_tx: usd_per_tx,
				recalibration_period: to_duration(self.args.arg_price_update_period.as_str())?,
				api_endpoint: self.args.arg_usd_per_eth.clone(),
			})
		}
	}

	pub fn extra_data(&self) -> Result<Bytes, String> {
		match self.args.arg_extra_data.as_ref() {
			Some(x) if x.len() <= 32 => Ok(x.as_bytes().to_owned()),
			None => Ok(version_data()),
			Some(_) => Err("Extra data must be at most 32 characters".into()),
		}
	}

	pub fn init_reserved_nodes(&self) -> Result<Vec<String>, String> {
		use std::fs::File;

		match self.args.arg_reserved_peers {
			Some(ref path) => {
				let path = replace_home(&self.directories().base, path);

				let mut buffer = String::new();
				let mut node_file = File::open(&path).map_err(|e| format!("Error opening reserved nodes file: {}", e))?;
				node_file.read_to_string(&mut buffer).map_err(|_| "Error reading reserved node file")?;
				let lines = buffer.lines().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty() && !s.starts_with("#")).collect::<Vec<_>>();

				for line in &lines {
					match validate_node_url(line).map(Into::into) {
						None => continue,
						Some(sync::Error::AddressResolve(_)) => return Err(format!("Failed to resolve hostname of a boot node: {}", line)),
						Some(_) => return Err(format!("Invalid node address format given for a boot node: {}", line)),
					}
				}

				Ok(lines)
			},
			None => Ok(Vec::new())
		}
	}

	pub fn net_addresses(&self) -> Result<(SocketAddr, Option<SocketAddr>), String> {
		let port = self.args.arg_ports_shift + self.args.arg_port;
		let listen_address = SocketAddr::new(self.interface(&self.args.arg_interface).parse().unwrap(), port);
		let public_address = if self.args.arg_nat.starts_with("extip:") {
			let host = self.args.arg_nat[6..].split(':').next().expect("split has at least one part; qed");
			let host = format!("{}:{}", host, port);
			match host.to_socket_addrs() {
				Ok(mut addr_iter) => {
					if let Some(addr) = addr_iter.next() {
						Some(addr)
					} else {
						return Err(format!("Invalid host given with `--nat extip:{}`", &self.args.arg_nat[6..]))
					}
				},
				Err(_) => return Err(format!("Invalid host given with `--nat extip:{}`", &self.args.arg_nat[6..]))
			}
		} else {
			None
		};
		Ok((listen_address, public_address))
	}

	pub fn net_config(&self) -> Result<NetworkConfiguration, String> {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.arg_nat == "any" || self.args.arg_nat == "upnp" || self.args.arg_nat == "natpmp";
		ret.nat_type = match &self.args.arg_nat[..] {
			"any" => NatType::Any,
			"upnp" => NatType::UPnP,
			"natpmp" => NatType::NatPMP,
			_ => NatType::Nothing,
		};
		ret.boot_nodes = to_bootnodes(&self.args.arg_bootnodes)?;
		let (listen, public) = self.net_addresses()?;
		ret.listen_address = Some(format!("{}", listen));
		ret.public_address = public.map(|p| format!("{}", p));
		ret.use_secret = match self.args.arg_node_key.as_ref()
			.map(|s| s.parse::<Secret>().or_else(|_| Secret::import_key(keccak(s).as_bytes())).map_err(|e| format!("Invalid key: {:?}", e))
			) {
			None => None,
			Some(Ok(key)) => Some(key),
			Some(Err(err)) => return Err(err),
		};
		ret.discovery_enabled = !self.args.flag_no_discovery;
		ret.max_peers = self.max_peers();
		ret.min_peers = self.min_peers();
		ret.snapshot_peers = self.snapshot_peers();
		ret.ip_filter = self.ip_filter()?;
		ret.max_pending_peers = self.max_pending_peers();
		let mut net_path = PathBuf::from(self.directories().base);
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret.reserved_nodes = self.init_reserved_nodes()?;
		ret.allow_non_reserved = !self.args.flag_reserved_only;
		ret.client_version = {
			let mut client_version = version();
			if !self.args.arg_identity.is_empty() {
				// Insert name after the "OpenEthereum/" at the beginning of version string.
				let idx = client_version.find('/').unwrap_or(client_version.len());
				client_version.insert_str(idx, &format!("/{}", self.args.arg_identity));
			}
			client_version
		};
		Ok(ret)
	}

	pub fn network_id(&self) -> Option<u64> {
		self.args.arg_network_id
	}

	pub fn rpc_apis(&self) -> String {
		let mut apis: Vec<&str> = self.args.arg_jsonrpc_apis.split(",").collect();

		if self.args.flag_geth {
			apis.insert(0, "personal");
		}

		apis.join(",")
	}

	pub fn cors(cors: &str) -> Option<Vec<String>> {
		match cors {
			"none" => return Some(Vec::new()),
			"*" | "all" | "any" => return None,
			_ => {},
		}

		Some(cors.split(',').map(Into::into).collect())
	}

	pub fn rpc_cors(&self) -> Option<Vec<String>> {
		let cors = self.args.arg_jsonrpc_cors.to_owned();
		Self::cors(&cors)
	}

	fn hosts(&self, hosts: &str, interface: &str) -> Option<Vec<String>> {
		if self.args.flag_unsafe_expose {
			return None;
		}

		if interface == "0.0.0.0" && hosts == "none" {
			return None;
		}

		Self::parse_hosts(hosts)
	}

	fn parse_hosts(hosts: &str) -> Option<Vec<String>> {
		match hosts {
			"none" => return Some(Vec::new()),
			"*" | "all" | "any" => return None,
			_ => {}
		}
		let hosts = hosts.split(',').map(Into::into).collect();
		Some(hosts)
	}

	pub fn rpc_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_jsonrpc_hosts, &self.rpc_interface())
	}

	fn ws_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_ws_hosts, &self.ws_interface())
	}

	fn ws_origins(&self) -> Option<Vec<String>> {
		if self.args.flag_unsafe_expose {
			return None;
		}

		Self::parse_hosts(&self.args.arg_ws_origins)
	}

	fn ipc_config(&self) -> Result<IpcConfiguration, String> {
		let conf = IpcConfiguration {
			chmod: self.args.arg_ipc_chmod.clone(),
			enabled: !self.args.flag_no_ipc,
			socket_addr: self.ipc_path(),
			apis: {
				let mut apis = self.args.arg_ipc_apis.clone();
				if self.args.flag_geth {
					if !apis.is_empty() {
						apis.push_str(",");
					}
					apis.push_str("personal");
				}
				apis.parse()?
			},
		};

		Ok(conf)
	}

	pub fn http_config(&self) -> Result<HttpConfiguration, String> {
		let mut conf = HttpConfiguration::default();
		conf.enabled = self.rpc_enabled();
		conf.interface = self.rpc_interface();
		conf.port = self.args.arg_ports_shift + self.args.arg_jsonrpc_port;
		conf.apis = self.rpc_apis().parse()?;
		conf.hosts = self.rpc_hosts();
		conf.cors = self.rpc_cors();
		if let Some(threads) = self.args.arg_jsonrpc_server_threads {
			conf.server_threads = std::cmp::max(1, threads);
		}
		if let Some(max_payload) = self.args.arg_jsonrpc_max_payload {
			conf.max_payload = std::cmp::max(1, max_payload);
		}
		conf.keep_alive = !self.args.flag_jsonrpc_no_keep_alive;

		Ok(conf)
	}

	pub fn ws_config(&self) -> Result<WsConfiguration, String> {
		let support_token_api =
			// enabled when not unlocking
			self.args.arg_unlock.is_none() && self.args.arg_enable_signing_queue;

		let conf = WsConfiguration {
			enabled: self.ws_enabled(),
			interface: self.ws_interface(),
			port: self.args.arg_ports_shift + self.args.arg_ws_port,
			apis: self.args.arg_ws_apis.parse()?,
			hosts: self.ws_hosts(),
			origins: self.ws_origins(),
			signer_path: self.directories().signer.into(),
			support_token_api,
			max_connections: self.args.arg_ws_max_connections,
		};

		Ok(conf)
	}

	pub fn private_provider_config(&self) -> Result<(ProviderConfig, EncryptorConfig, bool), String> {
		let dirs = self.directories();
		let provider_conf = ProviderConfig {
			validator_accounts: to_addresses(&self.args.arg_private_validators)?,
			signer_account: self.args.arg_private_signer.clone().and_then(|account| to_address(Some(account)).ok()),
			logs_path: match self.args.flag_private_enabled {
				true => Some(dirs.base),
				false => None,
			},
			use_offchain_storage: self.args.flag_private_state_offchain,
		};

		let encryptor_conf = EncryptorConfig {
			base_url: self.args.arg_private_sstore_url.clone(),
			threshold: self.args.arg_private_sstore_threshold.unwrap_or(0),
			key_server_account: self.args.arg_private_account.clone().and_then(|account| to_address(Some(account)).ok()),
		};

		Ok((provider_conf, encryptor_conf, self.args.flag_private_enabled))
	}

	pub fn snapshot_config(&self) -> Result<SnapshotConfiguration, String> {
		let mut conf = SnapshotConfiguration::default();
		conf.no_periodic = self.args.flag_no_periodic_snapshot;
		if let Some(threads) = self.args.arg_snapshot_threads {
			if threads > 0 {
				conf.processing_threads = threads;
			}
		}

		Ok(conf)
	}

	pub fn network_settings(&self) -> Result<NetworkSettings, String> {
		let http_conf = self.http_config()?;
		let net_addresses = self.net_addresses()?;
		Ok(NetworkSettings {
			name: self.args.arg_identity.clone(),
			chain: format!("{}", self.chain()?),
			is_dev_chain: self.is_dev_chain()?,
			network_port: net_addresses.0.port(),
			rpc_enabled: http_conf.enabled,
			rpc_interface: http_conf.interface,
			rpc_port: http_conf.port,
		})
	}

	pub fn update_policy(&self) -> Result<UpdatePolicy, String> {
		Ok(UpdatePolicy {
			enable_downloading: !self.args.flag_no_download,
			require_consensus: !self.args.flag_no_consensus,
			filter: match self.args.arg_auto_update.as_ref() {
				"none" => UpdateFilter::None,
				"critical" => UpdateFilter::Critical,
				"all" => UpdateFilter::All,
				_ => return Err("Invalid value for `--auto-update`. See `--help` for more information.".into()),
			},
			track: match self.args.arg_release_track.as_ref() {
				"stable" => ReleaseTrack::Stable,
				"nightly" => ReleaseTrack::Nightly,
				"current" => ReleaseTrack::Unknown,
				_ => return Err("Invalid value for `--releases-track`. See `--help` for more information.".into()),
			},
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: self.args.arg_auto_update_delay as u64,
			frequency: self.args.arg_auto_update_check_frequency as u64,
		})
	}

	pub fn directories(&self) -> Directories {
		let local_path = default_local_path();
		let default_data_path = default_data_path();
		let base_path: &str = match &self.args.arg_base_path {
			Some(x) => &x,
			None => &default_data_path,
		};
		let data_path = replace_home("", &(base_path));
		let is_using_base_path = self.args.arg_base_path.is_some();
		// If base_path is set and db_path is not we default to base path subdir instead of LOCAL.
		let base_db_path = if is_using_base_path && self.args.arg_db_path.is_none() {
			if self.args.flag_light {
				"$BASE/chains_light"
			} else {
				"$BASE/chains"
			}
		} else if self.args.flag_light {
			self.args.arg_db_path.as_ref().map_or(dir::CHAINS_PATH_LIGHT, |s| &s)
		} else {
			self.args.arg_db_path.as_ref().map_or(dir::CHAINS_PATH, |s| &s)
		};
		let cache_path = if is_using_base_path { "$BASE/cache" } else { dir::CACHE_PATH };

		let db_path = replace_home_and_local(&data_path, &local_path, &base_db_path);
		let cache_path = replace_home_and_local(&data_path, &local_path, cache_path);
		let keys_path = replace_home(&data_path, &self.args.arg_keys_path);
		let secretstore_path = replace_home(&data_path, &self.args.arg_secretstore_path);
		let ui_path = replace_home(&data_path, &self.args.arg_ui_path);

		Directories {
			keys: keys_path,
			base: data_path,
			cache: cache_path,
			db: db_path,
			signer: ui_path,
			secretstore: secretstore_path,
		}
	}

	pub fn ipc_path(&self) -> String {
			parity_ipc_path(
				&self.directories().base,
				&self.args.arg_ipc_path.clone(),
				self.args.arg_ports_shift,
			)
	}

	pub fn interface(&self, interface: &str) -> String {
		if self.args.flag_unsafe_expose {
			return "0.0.0.0".into();
		}

		match interface {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	pub fn rpc_interface(&self) -> String {
		let rpc_interface = self.args.arg_jsonrpc_interface.clone();
		self.interface(&rpc_interface)
	}

	pub fn ws_interface(&self) -> String {
		self.interface(&self.args.arg_ws_interface)
	}

	pub fn secretstore_interface(&self) -> String {
		self.interface(&self.args.arg_secretstore_interface)
	}

	pub fn secretstore_http_interface(&self) -> String {
		self.interface(&self.args.arg_secretstore_http_interface)
	}

	pub fn secretstore_cors(&self) -> Option<Vec<String>> {
		Self::cors(self.args.arg_secretstore_http_cors.as_ref())
	}

	fn secretstore_self_secret(&self) -> Result<Option<NodeSecretKey>, String> {
		match self.args.arg_secretstore_secret {
			Some(ref s) if s.len() == 64 => Ok(Some(NodeSecretKey::Plain(s.parse()
				.map_err(|e| format!("Invalid secret store secret: {}. Error: {:?}", s, e))?))),
			#[cfg(feature = "accounts")]
			Some(ref s) if s.len() == 40 => Ok(Some(NodeSecretKey::KeyStore(s.parse()
				.map_err(|e| format!("Invalid secret store secret address: {}. Error: {:?}", s, e))?))),
			Some(_) => Err(format!("Invalid secret store secret. Must be either existing account address, or hex-encoded private key")),
			None => Ok(None),
		}
	}

	fn secretstore_admin_public(&self) -> Result<Option<Public>, String> {
		match self.args.arg_secretstore_admin_public.as_ref() {
			Some(admin_public) => Ok(Some(admin_public.parse().map_err(|e| format!("Invalid secret store admin public: {}", e))?)),
			None => Ok(None),
		}
	}

	fn secretstore_nodes(&self) -> Result<BTreeMap<Public, (String, u16)>, String> {
		let mut nodes = BTreeMap::new();
		for node in self.args.arg_secretstore_nodes.split(',').filter(|n| n != &"") {
			let public_and_addr: Vec<_> = node.split('@').collect();
			if public_and_addr.len() != 2 {
				return Err(format!("Invalid secret store node: {}", node));
			}

			let ip_and_port: Vec<_> = public_and_addr[1].split(':').collect();
			if ip_and_port.len() != 2 {
				return Err(format!("Invalid secret store node: {}", node));
			}

			let public = public_and_addr[0].parse()
				.map_err(|e| format!("Invalid public key in secret store node: {}. Error: {:?}", public_and_addr[0], e))?;
			let port = ip_and_port[1].parse()
				.map_err(|e| format!("Invalid port in secret store node: {}. Error: {:?}", ip_and_port[1], e))?;

			nodes.insert(public, (ip_and_port[0].into(), port));
		}

		Ok(nodes)
	}

	fn stratum_interface(&self) -> String {
		self.interface(&self.args.arg_stratum_interface)
	}

	fn rpc_enabled(&self) -> bool {
		!self.args.flag_no_jsonrpc
	}

	fn ws_enabled(&self) -> bool {
		!self.args.flag_no_ws
	}

	fn secretstore_enabled(&self) -> bool {
		!self.args.flag_no_secretstore && cfg!(feature = "secretstore")
	}

	fn secretstore_http_enabled(&self) -> bool {
		!self.args.flag_no_secretstore_http && cfg!(feature = "secretstore")
	}

	fn secretstore_auto_migrate_enabled(&self) -> bool {
		!self.args.flag_no_secretstore_auto_migrate
	}

	fn secretstore_acl_check_contract_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_acl_contract.as_ref())
	}

	fn secretstore_service_contract_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_contract.as_ref())
	}

	fn secretstore_service_contract_srv_gen_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_srv_gen_contract.as_ref())
	}

	fn secretstore_service_contract_srv_retr_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_srv_retr_contract.as_ref())
	}

	fn secretstore_service_contract_doc_store_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_doc_store_contract.as_ref())
	}

	fn secretstore_service_contract_doc_sretr_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_doc_sretr_contract.as_ref())
	}

	fn secretstore_key_server_set_contract_address(&self) -> Result<Option<SecretStoreContractAddress>, String> {
		into_secretstore_service_contract_address(self.args.arg_secretstore_server_set_contract.as_ref())
	}

	fn verifier_settings(&self) -> VerifierSettings {
		let mut settings = VerifierSettings::default();
		settings.scale_verifiers = self.args.flag_scale_verifiers;
		if let Some(num_verifiers) = self.args.arg_num_verifiers {
			settings.num_verifiers = num_verifiers;
		}

		settings
	}
}

fn into_secretstore_service_contract_address(s: Option<&String>) -> Result<Option<SecretStoreContractAddress>, String> {
	match s.map(String::as_str) {
		None | Some("none") => Ok(None),
		Some("registry") => Ok(Some(SecretStoreContractAddress::Registry)),
		Some(a) => Ok(Some(SecretStoreContractAddress::Address(a.parse().map_err(|e| format!("{}", e))?))),
	}
}
