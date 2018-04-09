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

use std::cmp::{max, min};
use std::time::Duration;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::str::FromStr;
use cli::{Args, ArgsError};
use hash::keccak;
use ethereum_types::{U256, H256, Address};
use parity_version::{version_data, version};
use bytes::Bytes;
use ansi_term::Colour;
use ethsync::{NetworkConfiguration, validate_node_url, self};
use ethcore::ethstore::ethkey::{Secret, Public};
use ethcore::client::{VMType};
use ethcore::miner::{MinerOptions, Banning, StratumOptions};
use ethcore::verification::queue::VerifierSettings;

use rpc::{IpcConfiguration, HttpConfiguration, WsConfiguration, UiConfiguration};
use rpc_apis::ApiSet;
use parity_rpc::NetworkSettings;
use cache::CacheConfig;
use helpers::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_price, geth_ipc_path, parity_ipc_path,
to_bootnodes, to_addresses, to_address, to_gas_limit, to_queue_strategy, passwords_from_files};
use dir::helpers::{replace_home, replace_home_and_local};
use params::{ResealPolicy, AccountsConfig, GasPricerConfig, MinerExtras, SpecType};
use ethcore_logger::Config as LogConfig;
use dir::{self, Directories, default_hypervisor_path, default_local_path, default_data_path};
use dapps::Configuration as DappsConfiguration;
use ipfs::Configuration as IpfsConfiguration;
use ethcore_private_tx::{ProviderConfig, EncryptorConfig};
use secretstore::{NodeSecretKey, Configuration as SecretStoreConfiguration, ContractAddress as SecretStoreContractAddress};
use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};
use run::RunCmd;
use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, KillBlockchain, ExportState, DataFormat};
use export_hardcoded_sync::ExportHsyncCmd;
use presale::ImportWallet;
use account::{AccountCmd, NewAccount, ListAccounts, ImportAccounts, ImportFromGethAccounts};
use snapshot::{self, SnapshotCommand};
use network::{IpFilter};

const DEFAULT_MAX_PEERS: u16 = 50;
const DEFAULT_MIN_PEERS: u16 = 25;

#[derive(Debug, PartialEq)]
pub enum Cmd {
	Run(RunCmd),
	Version,
	Account(AccountCmd),
	ImportPresaleWallet(ImportWallet),
	Blockchain(BlockchainCmd),
	SignerToken(WsConfiguration, UiConfiguration, LogConfig),
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

pub struct Execute {
	pub logger: LogConfig,
	pub cmd: Cmd,
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
	pub args: Args,
	pub spec_name_override: Option<String>,
}

impl Configuration {
	pub fn parse<S: AsRef<str>>(command: &[S], spec_name_override: Option<String>) -> Result<Self, ArgsError> {
		let args = Args::parse(command)?;

		let config = Configuration {
			args: args,
			spec_name_override: spec_name_override,
		};

		Ok(config)
	}

	pub fn into_command(self) -> Result<Execute, String> {
		let dirs = self.directories();
		let pruning = self.args.arg_pruning.parse()?;
		let pruning_history = self.args.arg_pruning_history;
		let vm_type = self.vm_type()?;
		let spec = self.chain()?;
		let mode = match self.args.arg_mode.as_ref() {
			"last" => None,
			mode => Some(to_mode(&mode, self.args.arg_mode_timeout, self.args.arg_mode_alarm)?),
		};
		let update_policy = self.update_policy()?;
		let logger_config = self.logger_config();
		let ws_conf = self.ws_config()?;
		let http_conf = self.http_config()?;
		let ipc_conf = self.ipc_config()?;
		let net_conf = self.net_config()?;
		let ui_conf = self.ui_config();
		let network_id = self.network_id();
		let cache_config = self.cache_config();
		let tracing = self.args.arg_tracing.parse()?;
		let fat_db = self.args.arg_fat_db.parse()?;
		let compaction = self.args.arg_db_compaction.parse()?;
		let wal = !self.args.flag_fast_and_loose;
		let public_node = self.args.flag_public_node;
		let warp_sync = !self.args.flag_no_warp;
		let geth_compatibility = self.args.flag_geth;
		let dapps_conf = self.dapps_config();
		let ipfs_conf = self.ipfs_config();
		let secretstore_conf = self.secretstore_config()?;
		let format = self.format()?;

		let cmd = if self.args.flag_version {
			Cmd::Version
		} else if self.args.cmd_signer {
			let authfile = ::signer::codes_path(&ws_conf.signer_path);

			if self.args.cmd_signer_new_token {
				Cmd::SignerToken(ws_conf, ui_conf, logger_config.clone())
			} else if self.args.cmd_signer_sign {
				let pwfile = self.args.arg_password.first().map(|pwfile| {
					PathBuf::from(pwfile)
				});
				Cmd::SignerSign {
					id: self.args.arg_signer_sign_id,
					pwfile: pwfile,
					port: ws_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_signer_reject  {
				Cmd::SignerReject {
					id: self.args.arg_signer_reject_id,
					port: ws_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_signer_list  {
				Cmd::SignerList {
					port: ws_conf.port,
					authfile: authfile,
				}
			} else {
				unreachable!();
			}
		} else if self.args.cmd_tools && self.args.cmd_tools_hash {
			Cmd::Hash(self.args.arg_tools_hash_file)
		} else if self.args.cmd_db && self.args.cmd_db_kill {
			Cmd::Blockchain(BlockchainCmd::Kill(KillBlockchain {
				spec: spec,
				dirs: dirs,
				pruning: pruning,
			}))
		} else if self.args.cmd_account {
			let account_cmd = if self.args.cmd_account_new {
				let new_acc = NewAccount {
					iterations: self.args.arg_keys_iterations,
					path: dirs.keys,
					spec: spec,
					password_file: self.args.arg_password.first().map(|x| x.to_owned()),
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
		} else if self.args.flag_import_geth_keys {
        	let account_cmd = AccountCmd::ImportFromGeth(
				ImportFromGethAccounts {
					spec: spec,
					to: dirs.keys,
					testnet: self.args.flag_testnet
				}
			);
			Cmd::Account(account_cmd)
		} else if self.args.cmd_wallet {
			let presale_cmd = ImportWallet {
				iterations: self.args.arg_keys_iterations,
				path: dirs.keys,
				spec: spec,
				wallet_path: self.args.arg_wallet_import_path.unwrap().clone(),
				password_file: self.args.arg_password.first().map(|x| x.to_owned()),
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
				wal: wal,
				tracing: tracing,
				fat_db: fat_db,
				vm_type: vm_type,
				check_seal: !self.args.flag_no_seal_check,
				with_color: logger_config.color,
				verifier_settings: self.verifier_settings(),
				light: self.args.flag_light,
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
					wal: wal,
					tracing: tracing,
					fat_db: fat_db,
					from_block: to_block_id(&self.args.arg_export_blocks_from)?,
					to_block: to_block_id(&self.args.arg_export_blocks_to)?,
					check_seal: !self.args.flag_no_seal_check,
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
					wal: wal,
					tracing: tracing,
					fat_db: fat_db,
					at: to_block_id(&self.args.arg_export_state_at)?,
					storage: !self.args.flag_export_state_no_storage,
					code: !self.args.flag_export_state_no_code,
					min_balance: self.args.arg_export_state_min_balance.and_then(|s| to_u256(&s).ok()),
					max_balance: self.args.arg_export_state_max_balance.and_then(|s| to_u256(&s).ok()),
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
				wal: wal,
				kind: snapshot::Kind::Take,
				block_at: to_block_id(&self.args.arg_snapshot_at)?,
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
				wal: wal,
				kind: snapshot::Kind::Restore,
				block_at: to_block_id("latest")?, // unimportant.
			};
			Cmd::Snapshot(restore_cmd)
		} else if self.args.cmd_export_hardcoded_sync {
			let export_hs_cmd = ExportHsyncCmd {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				compaction: compaction,
				wal: wal,
			};
			Cmd::ExportHardcodedSync(export_hs_cmd)
		} else {
			let daemon = if self.args.cmd_daemon {
				Some(self.args.arg_daemon_pid_file.clone().expect("CLI argument is required; qed"))
			} else {
				None
			};

			let verifier_settings = self.verifier_settings();
			let whisper_config = self.whisper_config();
			let (private_provider_conf, private_enc_conf, private_tx_enabled) = self.private_provider_config()?;

			let run_cmd = RunCmd {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.arg_pruning_memory,
				daemon: daemon,
				logger_config: logger_config.clone(),
				miner_options: self.miner_options()?,
				gas_price_percentile: self.args.arg_gas_price_percentile,
				ntp_servers: self.ntp_servers(),
				ws_conf: ws_conf,
				http_conf: http_conf,
				ipc_conf: ipc_conf,
				net_conf: net_conf,
				network_id: network_id,
				acc_conf: self.accounts_config()?,
				gas_pricer_conf: self.gas_pricer_config()?,
				miner_extras: self.miner_extras()?,
				stratum: self.stratum_options()?,
				update_policy: update_policy,
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				wal: wal,
				vm_type: vm_type,
				warp_sync: warp_sync,
				warp_barrier: self.args.arg_warp_barrier,
				public_node: public_node,
				geth_compatibility: geth_compatibility,
				net_settings: self.network_settings()?,
				dapps_conf: dapps_conf,
				ipfs_conf: ipfs_conf,
				ui_conf: ui_conf,
				secretstore_conf: secretstore_conf,
				private_provider_conf: private_provider_conf,
				private_encryptor_conf: private_enc_conf,
				private_tx_enabled,
				dapp: self.dapp_to_open()?,
				ui: self.args.cmd_ui,
				name: self.args.arg_identity,
				custom_bootnodes: self.args.arg_bootnodes.is_some(),
				no_periodic_snapshot: self.args.flag_no_periodic_snapshot,
				check_seal: !self.args.flag_no_seal_check,
				download_old_blocks: !self.args.flag_no_ancient_blocks,
				verifier_settings: verifier_settings,
				serve_light: !self.args.flag_no_serve_light,
				light: self.args.flag_light,
				no_persistent_txqueue: self.args.flag_no_persistent_txqueue,
				whisper: whisper_config,
				no_hardcoded_sync: self.args.flag_no_hardcoded_sync,
			};
			Cmd::Run(run_cmd)
		};

		Ok(Execute {
			logger: logger_config,
			cmd: cmd,
		})
	}

	fn vm_type(&self) -> Result<VMType, String> {
		Ok(VMType::Interpreter)
	}

	fn miner_extras(&self) -> Result<MinerExtras, String> {
		let extras = MinerExtras {
			author: self.author()?,
			extra_data: self.extra_data()?,
			gas_floor_target: to_u256(&self.args.arg_gas_floor_target)?,
			gas_ceil_target: to_u256(&self.args.arg_gas_cap)?,
			engine_signer: self.engine_signer()?,
		};

		Ok(extras)
	}

	fn author(&self) -> Result<Address, String> {
		to_address(self.args.arg_etherbase.clone().or(self.args.arg_author.clone()))
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
		match self.args.arg_cache_size.or(self.args.arg_cache) {
			Some(size) => CacheConfig::new_with_total_cache_size(size),
			None => CacheConfig::new(
				self.args.arg_cache_size_db,
				self.args.arg_cache_size_blocks,
				self.args.arg_cache_size_queue,
				self.args.arg_cache_size_state,
			),
		}
	}

	fn logger_config(&self) -> LogConfig {
		LogConfig {
			mode: self.args.arg_logging.clone(),
			color: !self.args.flag_no_color && !cfg!(windows),
			file: self.args.arg_log_file.clone(),
		}
	}

	fn chain(&self) -> Result<SpecType, String> {
		let name = if let Some(ref s) = self.spec_name_override {
			s.clone()
		} else if self.args.flag_testnet {
			"testnet".to_owned()
		} else {
			self.args.arg_chain.clone()
		};

		Ok(name.parse()?)
	}

	fn max_peers(&self) -> u32 {
		self.args.arg_max_peers
			.or(max(self.args.arg_min_peers, Some(DEFAULT_MAX_PEERS)))
			.unwrap_or(DEFAULT_MAX_PEERS) as u32
	}

	fn ip_filter(&self) -> Result<IpFilter, String> {
		match IpFilter::parse(self.args.arg_allow_ips.as_str()) {
			Ok(allow_ip) => Ok(allow_ip),
			Err(_) => Err("Invalid IP filter value".to_owned()),
		}
	}

	fn min_peers(&self) -> u32 {
		self.args.arg_min_peers
			.or(min(self.args.arg_max_peers, Some(DEFAULT_MIN_PEERS)))
			.unwrap_or(DEFAULT_MIN_PEERS) as u32
	}

	fn max_pending_peers(&self) -> u32 {
		self.args.arg_max_pending_peers as u32
	}

	fn snapshot_peers(&self) -> u32 {
		self.args.arg_snapshot_peers as u32
	}

	fn work_notify(&self) -> Vec<String> {
		self.args.arg_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	fn accounts_config(&self) -> Result<AccountsConfig, String> {
		let cfg = AccountsConfig {
			iterations: self.args.arg_keys_iterations,
			refresh_time: self.args.arg_accounts_refresh,
			testnet: self.args.flag_testnet,
			password_files: self.args.arg_password.clone(),
			unlocked_accounts: to_addresses(&self.args.arg_unlock)?,
			enable_hardware_wallets: !self.args.flag_no_hardware_wallets,
			enable_fast_unlock: self.args.flag_fast_unlock,
		};

		Ok(cfg)
	}

	fn stratum_options(&self) -> Result<Option<StratumOptions>, String> {
		if self.args.flag_stratum {
			Ok(Some(StratumOptions {
				io_path: self.directories().db,
				listen_addr: self.stratum_interface(),
				port: self.args.arg_ports_shift + self.args.arg_stratum_port,
				secret: self.args.arg_stratum_secret.as_ref().map(|s| s.parse::<H256>().unwrap_or_else(|_| keccak(s))),
			}))
		} else { Ok(None) }
	}

	fn miner_options(&self) -> Result<MinerOptions, String> {
		let is_dev_chain = self.chain()? == SpecType::Dev;
		if is_dev_chain && self.args.flag_force_sealing && self.args.arg_reseal_min_period == 0 {
			return Err("Force sealing can't be used with reseal_min_period = 0".into());
		}

		let reseal = self.args.arg_reseal_on_txs.parse::<ResealPolicy>()?;

		let options = MinerOptions {
			new_work_notify: self.work_notify(),
			force_sealing: self.args.flag_force_sealing,
			reseal_on_external_tx: reseal.external,
			reseal_on_own_tx: reseal.own,
			reseal_on_uncle: self.args.flag_reseal_on_uncle,
			tx_gas_limit: match self.args.arg_tx_gas_limit {
				Some(ref d) => to_u256(d)?,
				None => U256::max_value(),
			},
			tx_queue_size: self.args.arg_tx_queue_size,
			tx_queue_memory_limit: if self.args.arg_tx_queue_mem_limit > 0 {
				Some(self.args.arg_tx_queue_mem_limit as usize * 1024 * 1024)
			} else { None },
			tx_queue_gas_limit: to_gas_limit(&self.args.arg_tx_queue_gas)?,
			tx_queue_strategy: to_queue_strategy(&self.args.arg_tx_queue_strategy)?,
			pending_set: to_pending_set(&self.args.arg_relay_set)?,
			reseal_min_period: Duration::from_millis(self.args.arg_reseal_min_period),
			reseal_max_period: Duration::from_millis(self.args.arg_reseal_max_period),
			work_queue_size: self.args.arg_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
			tx_queue_banning: match self.args.arg_tx_time_limit {
				Some(limit) => Banning::Enabled {
					min_offends: self.args.arg_tx_queue_ban_count,
					offend_threshold: Duration::from_millis(limit),
					ban_duration: Duration::from_secs(self.args.arg_tx_queue_ban_time as u64),
				},
				None => Banning::Disabled,
			},
			refuse_service_transactions: self.args.flag_refuse_service_transactions,
			infinite_pending_block: self.args.flag_infinite_pending_block,
		};

		Ok(options)
	}

	fn ui_port(&self) -> u16 {
		self.args.arg_ports_shift + self.args.arg_ui_port
	}

	fn ntp_servers(&self) -> Vec<String> {
		self.args.arg_ntp_servers.split(",").map(str::to_owned).collect()
	}

	fn ui_config(&self) -> UiConfiguration {
		UiConfiguration {
			enabled: self.ui_enabled(),
			interface: self.ui_interface(),
			port: self.ui_port(),
			hosts: self.ui_hosts(),
		}
	}

	fn dapps_config(&self) -> DappsConfiguration {
		let dev_ui = if self.args.flag_ui_no_validation { vec![("127.0.0.1".to_owned(), 3000)] } else { vec![] };
		let ui_port = self.ui_port();

		DappsConfiguration {
			enabled: self.dapps_enabled(),
			dapps_path: PathBuf::from(self.directories().dapps),
			extra_dapps: if self.args.cmd_dapp {
				self.args.arg_dapp_path.iter().map(|path| PathBuf::from(path)).collect()
			} else {
				vec![]
			},
			extra_embed_on: {
				let mut extra_embed = dev_ui.clone();
				match self.ui_hosts() {
					// In case host validation is disabled allow all frame ancestors
					None => {
						// NOTE Chrome does not seem to support "*:<port>"
						// we use `http(s)://*:<port>` instead.
						extra_embed.push(("http://*".to_owned(), ui_port));
						extra_embed.push(("https://*".to_owned(), ui_port));
					},
					Some(hosts) => extra_embed.extend(hosts.into_iter().filter_map(|host| {
						let mut it = host.split(":");
						let host = it.next();
						let port = it.next().and_then(|v| u16::from_str(v).ok());

						match (host, port) {
							(Some(host), Some(port)) => Some((host.into(), port)),
							(Some(host), None) => Some((host.into(), ui_port)),
							_ => None,
						}
					})),
				}
				extra_embed
			},
			extra_script_src: dev_ui,
		}
	}

	fn secretstore_config(&self) -> Result<SecretStoreConfiguration, String> {
		Ok(SecretStoreConfiguration {
			enabled: self.secretstore_enabled(),
			http_enabled: self.secretstore_http_enabled(),
			acl_check_enabled: self.secretstore_acl_check_enabled(),
			auto_migrate_enabled: self.secretstore_auto_migrate_enabled(),
			service_contract_address: self.secretstore_service_contract_address()?,
			service_contract_srv_gen_address: self.secretstore_service_contract_srv_gen_address()?,
			service_contract_srv_retr_address: self.secretstore_service_contract_srv_retr_address()?,
			service_contract_doc_store_address: self.secretstore_service_contract_doc_store_address()?,
			service_contract_doc_sretr_address: self.secretstore_service_contract_doc_sretr_address()?,
			self_secret: self.secretstore_self_secret()?,
			nodes: self.secretstore_nodes()?,
			interface: self.secretstore_interface(),
			port: self.args.arg_ports_shift + self.args.arg_secretstore_port,
			http_interface: self.secretstore_http_interface(),
			http_port: self.args.arg_ports_shift + self.args.arg_secretstore_http_port,
			data_path: self.directories().secretstore,
			admin_public: self.secretstore_admin_public()?,
		})
	}

	fn ipfs_config(&self) -> IpfsConfiguration {
		IpfsConfiguration {
			enabled: self.args.flag_ipfs_api,
			port: self.args.arg_ports_shift + self.args.arg_ipfs_api_port,
			interface: self.ipfs_interface(),
			cors: self.ipfs_cors(),
			hosts: self.ipfs_hosts(),
		}
	}

	fn dapp_to_open(&self) -> Result<Option<String>, String> {
		if !self.args.cmd_dapp {
			return Ok(None);
		}
		let path = self.args.arg_dapp_path.as_ref().map(String::as_str).unwrap_or(".");
		let path = Path::new(path).canonicalize()
			.map_err(|e| format!("Invalid path: {}. Error: {:?}", path, e))?;
		let name = path.file_name()
			.and_then(|name| name.to_str())
			.ok_or_else(|| "Root path is not supported.".to_owned())?;
		Ok(Some(name.into()))
	}

	fn gas_pricer_config(&self) -> Result<GasPricerConfig, String> {
		fn wei_per_gas(usd_per_tx: f32, usd_per_eth: f32) -> U256 {
			let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
			let gas_per_tx: f32 = 21000.0;
			let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
			U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()
		}

		if let Some(dec) = self.args.arg_gasprice.as_ref() {
			return Ok(GasPricerConfig::Fixed(to_u256(dec)?));
		} else if let Some(dec) = self.args.arg_min_gas_price {
			return Ok(GasPricerConfig::Fixed(U256::from(dec)));
		} else if self.chain()? != SpecType::Foundation {
			return Ok(GasPricerConfig::Fixed(U256::zero()));
		}

		let usd_per_tx = to_price(&self.args.arg_usd_per_tx)?;
		if "auto" == self.args.arg_usd_per_eth.as_str() {
			// Just a very rough estimate to avoid accepting
			// ZGP transactions before the price is fetched
			// if user does not want it.
			let last_known_usd_per_eth = 10.0;
			return Ok(GasPricerConfig::Calibrated {
				initial_minimum: wei_per_gas(usd_per_tx, last_known_usd_per_eth),
				usd_per_tx: usd_per_tx,
				recalibration_period: to_duration(self.args.arg_price_update_period.as_str())?,
			});
		}

		let usd_per_eth = to_price(&self.args.arg_usd_per_eth)?;
		let wei_per_gas = wei_per_gas(usd_per_tx, usd_per_eth);

		info!(
			"Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)",
			Colour::White.bold().paint(format!("US${:.2}", usd_per_eth)),
			Colour::Yellow.bold().paint(format!("{}", wei_per_gas))
		);

		Ok(GasPricerConfig::Fixed(wei_per_gas))
	}

	fn extra_data(&self) -> Result<Bytes, String> {
		match self.args.arg_extradata.as_ref().or(self.args.arg_extra_data.as_ref()) {
			Some(x) if x.len() <= 32 => Ok(x.as_bytes().to_owned()),
			None => Ok(version_data()),
			Some(_) => Err("Extra data must be at most 32 characters".into()),
		}
	}

	fn init_reserved_nodes(&self) -> Result<Vec<String>, String> {
		use std::fs::File;

		match self.args.arg_reserved_peers {
			Some(ref path) => {
				let mut buffer = String::new();
				let mut node_file = File::open(path).map_err(|e| format!("Error opening reserved nodes file: {}", e))?;
				node_file.read_to_string(&mut buffer).map_err(|_| "Error reading reserved node file")?;
				let lines = buffer.lines().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty() && !s.starts_with("#")).collect::<Vec<_>>();

				for line in &lines {
					match validate_node_url(line).map(Into::into) {
						None => continue,
						Some(ethsync::ErrorKind::AddressResolve(_)) => return Err(format!("Failed to resolve hostname of a boot node: {}", line)),
						Some(_) => return Err(format!("Invalid node address format given for a boot node: {}", line)),
					}
				}

				Ok(lines)
			},
			None => Ok(Vec::new())
		}
	}

	fn net_addresses(&self) -> Result<(SocketAddr, Option<SocketAddr>), String> {
		let port = self.args.arg_ports_shift + self.args.arg_port;
		let listen_address = SocketAddr::new("0.0.0.0".parse().unwrap(), port);
		let public_address = if self.args.arg_nat.starts_with("extip:") {
			let host = &self.args.arg_nat[6..];
			let host = host.parse().map_err(|_| format!("Invalid host given with `--nat extip:{}`", host))?;
			Some(SocketAddr::new(host, port))
		} else {
			None
		};
		Ok((listen_address, public_address))
	}

	fn net_config(&self) -> Result<NetworkConfiguration, String> {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.arg_nat == "any" || self.args.arg_nat == "upnp";
		ret.boot_nodes = to_bootnodes(&self.args.arg_bootnodes)?;
		let (listen, public) = self.net_addresses()?;
		ret.listen_address = Some(format!("{}", listen));
		ret.public_address = public.map(|p| format!("{}", p));
		ret.use_secret = match self.args.arg_node_key.as_ref()
			.map(|s| s.parse::<Secret>().or_else(|_| Secret::from_unsafe_slice(&keccak(s))).map_err(|e| format!("Invalid key: {:?}", e))
			) {
			None => None,
			Some(Ok(key)) => Some(key),
			Some(Err(err)) => return Err(err),
		};
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
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
		ret.client_version = version();
		Ok(ret)
	}

	fn network_id(&self) -> Option<u64> {
		self.args.arg_network_id.or(self.args.arg_networkid)
	}

	fn rpc_apis(&self) -> String {
		let mut apis: Vec<&str> = self.args.arg_rpcapi
			.as_ref()
			.unwrap_or(&self.args.arg_jsonrpc_apis)
			.split(",")
			.collect();

		if self.args.flag_geth {
			apis.insert(0, "personal");
		}

		apis.join(",")
	}

	fn cors(cors: &str) -> Option<Vec<String>> {
		match cors {
			"none" => return Some(Vec::new()),
			"*" | "all" | "any" => return None,
			_ => {},
		}

		Some(cors.split(',').map(Into::into).collect())
	}

	fn rpc_cors(&self) -> Option<Vec<String>> {
		let cors = self.args.arg_rpccorsdomain.clone().unwrap_or_else(|| self.args.arg_jsonrpc_cors.to_owned());
		Self::cors(&cors)
	}

	fn ipfs_cors(&self) -> Option<Vec<String>> {
		Self::cors(self.args.arg_ipfs_api_cors.as_ref())
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

	fn ui_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_ui_hosts, &self.ui_interface())
	}

	fn rpc_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_jsonrpc_hosts, &self.rpc_interface())
	}

	fn ws_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_ws_hosts, &self.ws_interface())
	}

	fn ws_origins(&self) -> Option<Vec<String>> {
		if self.args.flag_unsafe_expose || self.args.flag_ui_no_validation {
			return None;
		}

		Self::parse_hosts(&self.args.arg_ws_origins)
	}

	fn ipfs_hosts(&self) -> Option<Vec<String>> {
		self.hosts(&self.args.arg_ipfs_api_hosts, &self.ipfs_interface())
	}

	fn ipc_config(&self) -> Result<IpcConfiguration, String> {
		let conf = IpcConfiguration {
			enabled: !(self.args.flag_ipcdisable || self.args.flag_ipc_off || self.args.flag_no_ipc),
			socket_addr: self.ipc_path(),
			apis: {
				let mut apis = self.args.arg_ipcapi.clone().unwrap_or(self.args.arg_ipc_apis.clone());
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

	fn http_config(&self) -> Result<HttpConfiguration, String> {
		let conf = HttpConfiguration {
			enabled: self.rpc_enabled(),
			interface: self.rpc_interface(),
			port: self.args.arg_ports_shift + self.args.arg_rpcport.unwrap_or(self.args.arg_jsonrpc_port),
			apis: match self.args.flag_public_node {
				false => self.rpc_apis().parse()?,
				true => self.rpc_apis().parse::<ApiSet>()?.retain(ApiSet::PublicContext),
			},
			hosts: self.rpc_hosts(),
			cors: self.rpc_cors(),
			server_threads: match self.args.arg_jsonrpc_server_threads {
				Some(threads) if threads > 0 => threads,
				_ => 1,
			},
			processing_threads: self.args.arg_jsonrpc_threads,
		};

		Ok(conf)
	}

	fn ws_config(&self) -> Result<WsConfiguration, String> {
		let ui = self.ui_config();
		let http = self.http_config()?;

		let support_token_api =
			// never enabled for public node
			!self.args.flag_public_node
			// enabled when not unlocking unless the ui is forced
			&& (self.args.arg_unlock.is_none() || ui.enabled);

		let conf = WsConfiguration {
			enabled: self.ws_enabled(),
			interface: self.ws_interface(),
			port: self.args.arg_ports_shift + self.args.arg_ws_port,
			apis: self.args.arg_ws_apis.parse()?,
			hosts: self.ws_hosts(),
			origins: self.ws_origins(),
			signer_path: self.directories().signer.into(),
			support_token_api,
			ui_address: ui.address(),
			dapps_address: http.address(),
			max_connections: self.args.arg_ws_max_connections,
		};

		Ok(conf)
	}

	fn private_provider_config(&self) -> Result<(ProviderConfig, EncryptorConfig, bool), String> {
		let provider_conf = ProviderConfig {
			validator_accounts: to_addresses(&self.args.arg_private_validators)?,
			signer_account: self.args.arg_private_signer.clone().and_then(|account| to_address(Some(account)).ok()),
			passwords: match self.args.arg_private_passwords.clone() {
				Some(file) => passwords_from_files(&vec![file].as_slice())?,
				None => Vec::new(),
			},
		};

		let encryptor_conf = EncryptorConfig {
			base_url: self.args.arg_private_sstore_url.clone(),
			threshold: self.args.arg_private_sstore_threshold.unwrap_or(0),
			key_server_account: self.args.arg_private_account.clone().and_then(|account| to_address(Some(account)).ok()),
			passwords: match self.args.arg_private_passwords.clone() {
				Some(file) => passwords_from_files(&vec![file].as_slice())?,
				None => Vec::new(),
			},
		};

		Ok((provider_conf, encryptor_conf, self.args.flag_private_enabled))
	}

	fn network_settings(&self) -> Result<NetworkSettings, String> {
		let http_conf = self.http_config()?;
		let net_addresses = self.net_addresses()?;
		Ok(NetworkSettings {
			name: self.args.arg_identity.clone(),
			chain: format!("{}", self.chain()?),
			network_port: net_addresses.0.port(),
			rpc_enabled: http_conf.enabled,
			rpc_interface: http_conf.interface,
			rpc_port: http_conf.port,
		})
	}

	fn update_policy(&self) -> Result<UpdatePolicy, String> {
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
				"beta" => ReleaseTrack::Beta,
				"nightly" => ReleaseTrack::Nightly,
				"testing" => ReleaseTrack::Testing,
				"current" => ReleaseTrack::Unknown,
				_ => return Err("Invalid value for `--releases-track`. See `--help` for more information.".into()),
			},
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: self.args.arg_auto_update_delay as u64,
			frequency: self.args.arg_auto_update_check_frequency as u64,
		})
	}

	fn directories(&self) -> Directories {
		let local_path = default_local_path();
		let base_path = self.args.arg_base_path.as_ref().or_else(|| self.args.arg_datadir.as_ref()).map_or_else(|| default_data_path(), |s| s.clone());
		let data_path = replace_home("", &base_path);
		let is_using_base_path = self.args.arg_base_path.is_some();
		// If base_path is set and db_path is not we default to base path subdir instead of LOCAL.
		let base_db_path = if is_using_base_path && self.args.arg_db_path.is_none() {
			"$BASE/chains"
		} else {
			self.args.arg_db_path.as_ref().map_or(dir::CHAINS_PATH, |s| &s)
		};
		let cache_path = if is_using_base_path { "$BASE/cache" } else { dir::CACHE_PATH };

		let db_path = replace_home_and_local(&data_path, &local_path, &base_db_path);
		let cache_path = replace_home_and_local(&data_path, &local_path, cache_path);
		let keys_path = replace_home(&data_path, &self.args.arg_keys_path);
		let dapps_path = replace_home(&data_path, &self.args.arg_dapps_path);
		let secretstore_path = replace_home(&data_path, &self.args.arg_secretstore_path);
		let ui_path = replace_home(&data_path, &self.args.arg_ui_path);

		Directories {
			keys: keys_path,
			base: data_path,
			cache: cache_path,
			db: db_path,
			dapps: dapps_path,
			signer: ui_path,
			secretstore: secretstore_path,
		}
	}

	fn ipc_path(&self) -> String {
		if self.args.flag_geth {
			geth_ipc_path(self.args.flag_testnet)
		} else {
			parity_ipc_path(
				&self.directories().base,
				&self.args.arg_ipcpath.clone().unwrap_or(self.args.arg_ipc_path.clone()),
				self.args.arg_ports_shift,
			)
		}
	}

	fn interface(&self, interface: &str) -> String {
		if self.args.flag_unsafe_expose {
			return "0.0.0.0".into();
		}

		match interface {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn ui_interface(&self) -> String {
		self.interface(&self.args.arg_ui_interface)
	}

	fn rpc_interface(&self) -> String {
		let rpc_interface = self.args.arg_rpcaddr.clone().unwrap_or(self.args.arg_jsonrpc_interface.clone());
		self.interface(&rpc_interface)
	}

	fn ws_interface(&self) -> String {
		self.interface(&self.args.arg_ws_interface)
	}

	fn ipfs_interface(&self) -> String {
		self.interface(&self.args.arg_ipfs_api_interface)
	}

	fn secretstore_interface(&self) -> String {
		self.interface(&self.args.arg_secretstore_interface)
	}

	fn secretstore_http_interface(&self) -> String {
		self.interface(&self.args.arg_secretstore_http_interface)
	}

	fn secretstore_self_secret(&self) -> Result<Option<NodeSecretKey>, String> {
		match self.args.arg_secretstore_secret {
			Some(ref s) if s.len() == 64 => Ok(Some(NodeSecretKey::Plain(s.parse()
				.map_err(|e| format!("Invalid secret store secret: {}. Error: {:?}", s, e))?))),
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
		!self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc
	}

	fn ws_enabled(&self) -> bool {
		!self.args.flag_no_ws
	}

	fn dapps_enabled(&self) -> bool {
		!self.args.flag_dapps_off && !self.args.flag_no_dapps && self.rpc_enabled() && cfg!(feature = "dapps")
	}

	fn secretstore_enabled(&self) -> bool {
		!self.args.flag_no_secretstore && cfg!(feature = "secretstore")
	}

	fn secretstore_http_enabled(&self) -> bool {
		!self.args.flag_no_secretstore_http && cfg!(feature = "secretstore")
	}

	fn secretstore_acl_check_enabled(&self) -> bool {
		!self.args.flag_no_secretstore_acl_check
	}

	fn secretstore_auto_migrate_enabled(&self) -> bool {
		!self.args.flag_no_secretstore_auto_migrate
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

	fn ui_enabled(&self) -> bool {
		if self.args.flag_force_ui {
			return true;
		}

		let ui_disabled = self.args.arg_unlock.is_some() ||
			self.args.flag_geth ||
			self.args.flag_no_ui;

		self.args.cmd_ui && !ui_disabled && cfg!(feature = "ui-enabled")
	}

	fn verifier_settings(&self) -> VerifierSettings {
		let mut settings = VerifierSettings::default();
		settings.scale_verifiers = self.args.flag_scale_verifiers;
		if let Some(num_verifiers) = self.args.arg_num_verifiers {
			settings.num_verifiers = num_verifiers;
		}

		settings
	}

	fn whisper_config(&self) -> ::whisper::Config {
		::whisper::Config {
			enabled: self.args.flag_whisper,
			target_message_pool_size: self.args.arg_whisper_pool_size * 1024 * 1024,
		}
	}
}

fn into_secretstore_service_contract_address(s: &str) -> Result<Option<SecretStoreContractAddress>, String> {
	match s {
		"none" => Ok(None),
		"registry" => Ok(Some(SecretStoreContractAddress::Registry)),
		a => Ok(Some(SecretStoreContractAddress::Address(a.parse().map_err(|e| format!("{}", e))?))),
	}
}

#[cfg(test)]
mod tests {
	use std::io::Write;
	use std::fs::File;
	use std::str::FromStr;

	use tempdir::TempDir;
	use ethcore::client::{VMType, BlockId};
	use ethcore::miner::MinerOptions;
	use miner::transaction_queue::PrioritizationStrategy;
	use parity_rpc::NetworkSettings;
	use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};

	use account::{AccountCmd, NewAccount, ImportAccounts, ListAccounts};
	use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, DataFormat, ExportState};
	use cli::Args;
	use dir::{Directories, default_hypervisor_path};
	use helpers::{default_network_config};
	use params::SpecType;
	use presale::ImportWallet;
	use rpc::{WsConfiguration, UiConfiguration};
	use run::RunCmd;

	use network::{AllowIP, IpFilter};

	extern crate ipnetwork;
	use self::ipnetwork::IpNetwork;

	use super::*;

	#[derive(Debug, PartialEq)]
	struct TestPasswordReader(&'static str);

	fn parse(args: &[&str]) -> Configuration {
		Configuration {
			args: Args::parse_without_config(args).unwrap(),
			spec_name_override: None,
		}
	}

	#[test]
	fn test_command_version() {
		let args = vec!["parity", "--version"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Version);
	}

	#[test]
	fn test_command_account_new() {
		let args = vec!["parity", "account", "new"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Account(AccountCmd::New(NewAccount {
			iterations: 10240,
			path: Directories::default().keys,
			password_file: None,
			spec: SpecType::default(),
		})));
	}

	#[test]
	fn test_command_account_list() {
		let args = vec!["parity", "account", "list"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Account(
			AccountCmd::List(ListAccounts {
				path: Directories::default().keys,
				spec: SpecType::default(),
			})
		));
	}

	#[test]
	fn test_command_account_import() {
		let args = vec!["parity", "account", "import", "my_dir", "another_dir"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Account(AccountCmd::Import(ImportAccounts {
			from: vec!["my_dir".into(), "another_dir".into()],
			to: Directories::default().keys,
			spec: SpecType::default(),
		})));
	}

	#[test]
	fn test_command_wallet_import() {
		let args = vec!["parity", "wallet", "import", "my_wallet.json", "--password", "pwd"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::ImportPresaleWallet(ImportWallet {
			iterations: 10240,
			path: Directories::default().keys,
			wallet_path: "my_wallet.json".into(),
			password_file: Some("pwd".into()),
			spec: SpecType::default(),
		}));
	}

	#[test]
	fn test_command_blockchain_import() {
		let args = vec!["parity", "import", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Blockchain(BlockchainCmd::Import(ImportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			format: Default::default(),
			pruning: Default::default(),
			pruning_history: 64,
			pruning_memory: 32,
			compaction: Default::default(),
			wal: true,
			tracing: Default::default(),
			fat_db: Default::default(),
			vm_type: VMType::Interpreter,
			check_seal: true,
			with_color: !cfg!(windows),
			verifier_settings: Default::default(),
			light: false,
		})));
	}

	#[test]
	fn test_command_blockchain_export() {
		let args = vec!["parity", "export", "blocks", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 64,
			pruning_memory: 32,
			format: Default::default(),
			compaction: Default::default(),
			wal: true,
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockId::Number(1),
			to_block: BlockId::Latest,
			check_seal: true,
		})));
	}

	#[test]
	fn test_command_state_export() {
		let args = vec!["parity", "export", "state", "state.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Blockchain(BlockchainCmd::ExportState(ExportState {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("state.json".into()),
			pruning: Default::default(),
			pruning_history: 64,
			pruning_memory: 32,
			format: Default::default(),
			compaction: Default::default(),
			wal: true,
			tracing: Default::default(),
			fat_db: Default::default(),
			at: BlockId::Latest,
			storage: true,
			code: true,
			min_balance: None,
			max_balance: None,
		})));
	}

	#[test]
	fn test_command_blockchain_export_with_custom_format() {
		let args = vec!["parity", "export", "blocks", "--format", "hex", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 64,
			pruning_memory: 32,
			format: Some(DataFormat::Hex),
			compaction: Default::default(),
			wal: true,
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockId::Number(1),
			to_block: BlockId::Latest,
			check_seal: true,
		})));
	}

	#[test]
	fn test_command_signer_new_token() {
		let args = vec!["parity", "signer", "new-token"];
		let conf = parse(&args);
		let expected = Directories::default().signer;
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::SignerToken(WsConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8546,
			apis: ApiSet::UnsafeContext,
			origins: Some(vec!["parity://*".into(),"chrome-extension://*".into(), "moz-extension://*".into()]),
			hosts: Some(vec![]),
			signer_path: expected.into(),
			ui_address: None,
			dapps_address: Some("127.0.0.1:8545".into()),
			support_token_api: true,
			max_connections: 100,
		}, UiConfiguration {
			enabled: false,
			interface: "127.0.0.1".into(),
			port: 8180,
			hosts: Some(vec![]),
		}, LogConfig {
            color: true,
            mode: None,
            file: None,
        } ));
	}

	#[test]
	fn test_ws_max_connections() {
		let args = vec!["parity", "--ws-max-connections", "1"];
		let conf = parse(&args);

		assert_eq!(conf.ws_config().unwrap(), WsConfiguration {
			max_connections: 1,
			..Default::default()
		});
	}

	#[test]
	fn test_run_cmd() {
		let args = vec!["parity"];
		let conf = parse(&args);
		let mut expected = RunCmd {
			cache_config: Default::default(),
			dirs: Default::default(),
			spec: Default::default(),
			pruning: Default::default(),
			pruning_history: 64,
			pruning_memory: 32,
			daemon: None,
			logger_config: Default::default(),
			miner_options: Default::default(),
			gas_price_percentile: 50,
			ntp_servers: vec![
				"0.parity.pool.ntp.org:123".into(),
				"1.parity.pool.ntp.org:123".into(),
				"2.parity.pool.ntp.org:123".into(),
				"3.parity.pool.ntp.org:123".into(),
			],
			ws_conf: Default::default(),
			http_conf: Default::default(),
			ipc_conf: Default::default(),
			net_conf: default_network_config(),
			network_id: None,
			public_node: false,
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
			wal: true,
			vm_type: Default::default(),
			geth_compatibility: false,
			net_settings: Default::default(),
			dapps_conf: Default::default(),
			ipfs_conf: Default::default(),
			ui_conf: Default::default(),
			secretstore_conf: Default::default(),
			private_provider_conf: Default::default(),
			private_encryptor_conf: Default::default(),
			private_tx_enabled: false,
			ui: false,
			dapp: None,
			name: "".into(),
			custom_bootnodes: false,
			fat_db: Default::default(),
			no_periodic_snapshot: false,
			stratum: None,
			check_seal: true,
			download_old_blocks: true,
			verifier_settings: Default::default(),
			serve_light: true,
			light: false,
			no_hardcoded_sync: false,
			no_persistent_txqueue: false,
			whisper: Default::default(),
		};
		expected.secretstore_conf.enabled = cfg!(feature = "secretstore");
		expected.secretstore_conf.http_enabled = cfg!(feature = "secretstore");
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::Run(expected));
	}

	#[test]
	fn should_parse_mining_options() {
		// given
		let mut mining_options = MinerOptions::default();

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--tx-queue-strategy", "gas_factor"]);
		let conf2 = parse(&["parity", "--tx-queue-strategy", "gas_price"]);
		let conf3 = parse(&["parity", "--tx-queue-strategy", "gas"]);

		// then
		assert_eq!(conf0.miner_options().unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasFactorAndGasPrice;
		assert_eq!(conf1.miner_options().unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasPriceOnly;
		assert_eq!(conf2.miner_options().unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasAndGasPrice;
		assert_eq!(conf3.miner_options().unwrap(), mining_options);
	}

	#[test]
	fn should_fail_on_force_reseal_and_reseal_min_period() {
		let conf = parse(&["parity", "--chain", "dev", "--force-sealing", "--reseal-min-period", "0"]);

		assert!(conf.miner_options().is_err());
	}

	#[test]
	fn should_parse_updater_options() {
		// when
		let conf0 = parse(&["parity", "--release-track=testing"]);
		let conf1 = parse(&["parity", "--auto-update", "all", "--no-consensus", "--auto-update-delay", "300"]);
		let conf2 = parse(&["parity", "--no-download", "--auto-update=all", "--release-track=beta", "--auto-update-delay=300", "--auto-update-check-frequency=100"]);
		let conf3 = parse(&["parity", "--auto-update=xxx"]);

		// then
		assert_eq!(conf0.update_policy().unwrap(), UpdatePolicy {
			enable_downloading: true,
			require_consensus: true,
			filter: UpdateFilter::Critical,
			track: ReleaseTrack::Testing,
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: 100,
			frequency: 20,
		});
		assert_eq!(conf1.update_policy().unwrap(), UpdatePolicy {
			enable_downloading: true,
			require_consensus: false,
			filter: UpdateFilter::All,
			track: ReleaseTrack::Unknown,
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: 300,
			frequency: 20,
		});
		assert_eq!(conf2.update_policy().unwrap(), UpdatePolicy {
			enable_downloading: false,
			require_consensus: true,
			filter: UpdateFilter::All,
			track: ReleaseTrack::Beta,
			path: default_hypervisor_path(),
			max_size: 128 * 1024 * 1024,
			max_delay: 300,
			frequency: 100,
		});
		assert!(conf3.update_policy().is_err());
	}

	#[test]
	fn should_parse_network_settings() {
		// given

		// when
		let conf = parse(&["parity", "--testnet", "--identity", "testname"]);

		// then
		assert_eq!(conf.network_settings(), Ok(NetworkSettings {
			name: "testname".to_owned(),
			chain: "kovan".to_owned(),
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

		// when
		let conf1 = parse(&["parity", "-j",
						 "--jsonrpc-port", "8000",
						 "--jsonrpc-interface", "all",
						 "--jsonrpc-cors", "*",
						 "--jsonrpc-apis", "web3,eth"
						 ]);
		let conf2 = parse(&["parity", "--rpc",
						  "--rpcport", "8000",
						  "--rpcaddr", "all",
						  "--rpccorsdomain", "*",
						  "--rpcapi", "web3,eth"
						  ]);

		// then
		assert(conf1);
		assert(conf2);
	}

	#[test]
	fn should_parse_rpc_hosts() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--jsonrpc-hosts", "none"]);
		let conf2 = parse(&["parity", "--jsonrpc-hosts", "all"]);
		let conf3 = parse(&["parity", "--jsonrpc-hosts", "parity.io,something.io"]);

		// then
		assert_eq!(conf0.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf1.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf2.rpc_hosts(), None);
		assert_eq!(conf3.rpc_hosts(), Some(vec!["parity.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_parse_ipfs_hosts() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--ipfs-api-hosts", "none"]);
		let conf2 = parse(&["parity", "--ipfs-api-hosts", "all"]);
		let conf3 = parse(&["parity", "--ipfs-api-hosts", "parity.io,something.io"]);

		// then
		assert_eq!(conf0.ipfs_hosts(), Some(Vec::new()));
		assert_eq!(conf1.ipfs_hosts(), Some(Vec::new()));
		assert_eq!(conf2.ipfs_hosts(), None);
		assert_eq!(conf3.ipfs_hosts(), Some(vec!["parity.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_parse_ipfs_cors() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--ipfs-api-cors", "*"]);
		let conf2 = parse(&["parity", "--ipfs-api-cors", "http://parity.io,http://something.io"]);

		// then
		assert_eq!(conf0.ipfs_cors(), Some(vec![]));
		assert_eq!(conf1.ipfs_cors(), None);
		assert_eq!(conf2.ipfs_cors(), Some(vec!["http://parity.io".into(),"http://something.io".into()]));
	}

	#[test]
	fn should_disable_signer_in_geth_compat() {
		// given

		// when
		let conf0 = parse(&["parity", "--geth"]);
		let conf1 = parse(&["parity", "--geth", "--force-ui"]);

		// then
		assert_eq!(conf0.ui_enabled(), false);
		assert_eq!(conf1.ui_enabled(), true);
	}

	#[test]
	fn should_disable_signer_when_account_is_unlocked() {
		// given

		// when
		let conf0 = parse(&["parity", "--unlock", "0x0"]);

		// then
		assert_eq!(conf0.ui_enabled(), false);
	}

	#[test]
	fn should_parse_ui_configuration() {
		// given

		// when
		let conf0 = parse(&["parity", "--ui-path=signer"]);
		let conf1 = parse(&["parity", "--ui-path=signer", "--ui-no-validation"]);
		let conf2 = parse(&["parity", "--ui-path=signer", "--ui-port", "3123"]);
		let conf3 = parse(&["parity", "--ui-path=signer", "--ui-interface", "test"]);
		let conf4 = parse(&["parity", "--ui-path=signer", "--force-ui"]);
		let conf5 = parse(&["parity", "--ui-path=signer", "ui"]);

		// then
		assert_eq!(conf0.directories().signer, "signer".to_owned());
		assert_eq!(conf0.ui_config(), UiConfiguration {
			enabled: false,
			interface: "127.0.0.1".into(),
			port: 8180,
			hosts: Some(vec![]),
		});
		assert!(conf4.ws_config().unwrap().hosts.is_some());
		assert_eq!(conf4.directories().signer, "signer".to_owned());
		assert_eq!(conf4.ui_config(), UiConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8180,
			hosts: Some(vec![]),
		});
		assert!(conf5.ws_config().unwrap().hosts.is_some());
		assert!(conf5.ws_config().unwrap().hosts.is_some());
		assert_eq!(conf5.directories().signer, "signer".to_owned());
		assert_eq!(conf5.ui_config(), UiConfiguration {
			enabled: true,
			interface: "127.0.0.1".into(),
			port: 8180,
			hosts: Some(vec![]),
		});
		assert!(conf5.ws_config().unwrap().hosts.is_some());
		assert_eq!(conf1.directories().signer, "signer".to_owned());
		assert_eq!(conf1.ui_config(), UiConfiguration {
			enabled: false,
			interface: "127.0.0.1".into(),
			port: 8180,
			hosts: Some(vec![]),
		});
		assert_eq!(conf1.dapps_config().extra_embed_on, vec![("127.0.0.1".to_owned(), 3000)]);
		assert_eq!(conf1.ws_config().unwrap().origins, None);
		assert_eq!(conf2.directories().signer, "signer".to_owned());
		assert_eq!(conf2.ui_config(), UiConfiguration {
			enabled: false,
			interface: "127.0.0.1".into(),
			port: 3123,
			hosts: Some(vec![]),
		});
		assert!(conf2.ws_config().unwrap().hosts.is_some());
		assert_eq!(conf3.directories().signer, "signer".to_owned());
		assert_eq!(conf3.ui_config(), UiConfiguration {
			enabled: false,
			interface: "test".into(),
			port: 8180,
			hosts: Some(vec![]),
		});
		assert!(conf3.ws_config().unwrap().hosts.is_some());
	}

	#[test]
	fn should_parse_dapp_opening() {
		// given
		let tempdir = TempDir::new("").unwrap();

		// when
		let conf0 = parse(&["parity", "dapp", tempdir.path().to_str().unwrap()]);

		// then
		assert_eq!(conf0.dapp_to_open(), Ok(Some(tempdir.path().file_name().unwrap().to_str().unwrap().into())));
		let extra_dapps = conf0.dapps_config().extra_dapps;
		assert_eq!(extra_dapps, vec![tempdir.path().to_owned()]);
	}

	#[test]
	fn should_not_bail_on_empty_line_in_reserved_peers() {
		let tempdir = TempDir::new("").unwrap();
		let filename = tempdir.path().join("peers");
		File::create(&filename).unwrap().write_all(b"  \n\t\n").unwrap();
		let args = vec!["parity", "--reserved-peers", filename.to_str().unwrap()];
		let conf = Configuration::parse(&args, None).unwrap();
		assert!(conf.init_reserved_nodes().is_ok());
	}

	#[test]
	fn should_ignore_comments_in_reserved_peers() {
		let tempdir = TempDir::new("").unwrap();
		let filename = tempdir.path().join("peers_comments");
		File::create(&filename).unwrap().write_all(b"# Sample comment\nenode://6f8a80d14311c39f35f516fa664deaaaa13e85b2f7493f37f6144d86991ec012937307647bd3b9a82abe2974e1407241d54947bbb39763a4cac9f77166ad92a0@172.0.0.1:30303\n").unwrap();
		let args = vec!["parity", "--reserved-peers", filename.to_str().unwrap()];
		let conf = Configuration::parse(&args, None).unwrap();
		let reserved_nodes = conf.init_reserved_nodes();
		assert!(reserved_nodes.is_ok());
		assert_eq!(reserved_nodes.unwrap().len(), 1);
	}

	#[test]
	fn test_dev_preset() {
		let args = vec!["parity", "--config", "dev"];
		let conf = Configuration::parse(&args, None).unwrap();
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
		let args = vec!["parity", "--config", "mining"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 50);
				assert_eq!(c.net_conf.max_peers, 100);
				assert_eq!(c.ipc_conf.enabled, false);
				assert_eq!(c.dapps_conf.enabled, false);
				assert_eq!(c.miner_options.force_sealing, true);
				assert_eq!(c.miner_options.reseal_on_external_tx, true);
				assert_eq!(c.miner_options.reseal_on_own_tx, true);
				assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(4000));
				assert_eq!(c.miner_options.tx_queue_size, 2048);
				assert_eq!(c.cache_config, CacheConfig::new_with_total_cache_size(256));
				assert_eq!(c.logger_config.mode.unwrap(), "miner=trace,own_tx=trace");
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_non_standard_ports_preset() {
		let args = vec!["parity", "--config", "non-standard-ports"];
		let conf = Configuration::parse(&args, None).unwrap();
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
		let args = vec!["parity", "--config", "insecure"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.update_policy.require_consensus, false);
				assert_eq!(c.net_settings.rpc_interface, "0.0.0.0");
				match c.http_conf.apis {
					ApiSet::List(set) => assert_eq!(set, ApiSet::All.list_apis()),
					_ => panic!("Incorrect rpc apis"),
				}
				// "web3,eth,net,personal,parity,parity_set,traces,rpc,parity_accounts");
				assert_eq!(c.http_conf.hosts, None);
				assert_eq!(c.ipfs_conf.hosts, None);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_dev_insecure_preset() {
		let args = vec!["parity", "--config", "dev-insecure"];
		let conf = Configuration::parse(&args, None).unwrap();
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
				assert_eq!(c.ipfs_conf.hosts, None);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn test_override_preset() {
		let args = vec!["parity", "--config", "mining", "--min-peers=99"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 99);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_apply_ports_shift() {
		// given

		// when
		let conf0 = parse(&["parity", "--ports-shift", "1", "--stratum"]);
		let conf1 = parse(&["parity", "--ports-shift", "1", "--jsonrpc-port", "8544"]);

		// then
		assert_eq!(conf0.net_addresses().unwrap().0.port(), 30304);
		assert_eq!(conf0.network_settings().unwrap().network_port, 30304);
		assert_eq!(conf0.network_settings().unwrap().rpc_port, 8546);
		assert_eq!(conf0.http_config().unwrap().port, 8546);
		assert_eq!(conf0.ws_config().unwrap().port, 8547);
		assert_eq!(conf0.ui_config().port, 8181);
		assert_eq!(conf0.secretstore_config().unwrap().port, 8084);
		assert_eq!(conf0.secretstore_config().unwrap().http_port, 8083);
		assert_eq!(conf0.ipfs_config().port, 5002);
		assert_eq!(conf0.stratum_options().unwrap().unwrap().port, 8009);


		assert_eq!(conf1.net_addresses().unwrap().0.port(), 30304);
		assert_eq!(conf1.network_settings().unwrap().network_port, 30304);
		assert_eq!(conf1.network_settings().unwrap().rpc_port, 8545);
		assert_eq!(conf1.http_config().unwrap().port, 8545);
		assert_eq!(conf1.ws_config().unwrap().port, 8547);
		assert_eq!(conf1.ui_config().port, 8181);
		assert_eq!(conf1.secretstore_config().unwrap().port, 8084);
		assert_eq!(conf1.secretstore_config().unwrap().http_port, 8083);
		assert_eq!(conf1.ipfs_config().port, 5002);
	}

	#[test]
	fn should_expose_all_servers() {
		// given

		// when
		let conf0 = parse(&["parity", "--unsafe-expose"]);

		// then
		assert_eq!(&conf0.network_settings().unwrap().rpc_interface, "0.0.0.0");
		assert_eq!(&conf0.http_config().unwrap().interface, "0.0.0.0");
		assert_eq!(conf0.http_config().unwrap().hosts, None);
		assert_eq!(&conf0.ws_config().unwrap().interface, "0.0.0.0");
		assert_eq!(conf0.ws_config().unwrap().hosts, None);
		assert_eq!(conf0.ws_config().unwrap().origins, None);
		assert_eq!(&conf0.ui_config().interface, "0.0.0.0");
		assert_eq!(conf0.ui_config().hosts, None);
		assert_eq!(&conf0.secretstore_config().unwrap().interface, "0.0.0.0");
		assert_eq!(&conf0.secretstore_config().unwrap().http_interface, "0.0.0.0");
		assert_eq!(&conf0.ipfs_config().interface, "0.0.0.0");
		assert_eq!(conf0.ipfs_config().hosts, None);
	}

	#[test]
	fn allow_ips() {
		let all = parse(&["parity", "--allow-ips", "all"]);
		let private = parse(&["parity", "--allow-ips", "private"]);
		let block_custom = parse(&["parity", "--allow-ips", "-10.0.0.0/8"]);
		let combo = parse(&["parity", "--allow-ips", "public 10.0.0.0/8 -1.0.0.0/8"]);
		let ipv6_custom_public = parse(&["parity", "--allow-ips", "public fc00::/7"]);
		let ipv6_custom_private = parse(&["parity", "--allow-ips", "private -fc00::/7"]);

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
		let std = parse(&["parity"]);
		let base = parse(&["parity", "--base-path", "/test"]);

		let base_path = ::dir::default_data_path();
		let local_path = ::dir::default_local_path();
		assert_eq!(std.directories().cache, dir::helpers::replace_home_and_local(&base_path, &local_path, ::dir::CACHE_PATH));
		assert_eq!(base.directories().cache, "/test/cache");
	}

	#[test]
	fn should_respect_only_max_peers_and_default() {
		let args = vec!["parity", "--max-peers=50"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 25);
				assert_eq!(c.net_conf.max_peers, 50);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_respect_only_max_peers_less_than_default() {
		let args = vec!["parity", "--max-peers=5"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 5);
				assert_eq!(c.net_conf.max_peers, 5);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}

	#[test]
	fn should_respect_only_min_peers_and_default() {
		let args = vec!["parity", "--min-peers=5"];
		let conf = Configuration::parse(&args, None).unwrap();
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
		let args = vec!["parity", "--min-peers=500"];
		let conf = Configuration::parse(&args, None).unwrap();
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.net_conf.min_peers, 500);
				assert_eq!(c.net_conf.max_peers, 500);
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}
}
