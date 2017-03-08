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

use std::time::Duration;
use std::io::{Read, Write, stderr};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::cmp::max;
use cli::{Args, ArgsError};
use util::{Hashable, H256, U256, Uint, Bytes, version_data, Address};
use util::journaldb::Algorithm;
use util::log::Colour;
use ethsync::{NetworkConfiguration, is_valid_node_url, AllowIP};
use ethcore::ethstore::ethkey::Secret;
use ethcore::client::{VMType};
use ethcore::miner::{MinerOptions, Banning, StratumOptions};
use ethcore::verification::queue::VerifierSettings;

use rpc::{IpcConfiguration, HttpConfiguration};
use ethcore_rpc::NetworkSettings;
use cache::CacheConfig;
use helpers::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_price, replace_home, replace_home_for_db,
geth_ipc_path, parity_ipc_path, to_bootnodes, to_addresses, to_address, to_gas_limit, to_queue_strategy};
use params::{SpecType, ResealPolicy, AccountsConfig, GasPricerConfig, MinerExtras, Pruning, Switch};
use ethcore_logger::Config as LogConfig;
use dir::{self, Directories, default_hypervisor_path, default_local_path, default_data_path};
use dapps::Configuration as DappsConfiguration;
use ipfs::Configuration as IpfsConfiguration;
use signer::{Configuration as SignerConfiguration};
use secretstore::Configuration as SecretStoreConfiguration;
use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};
use run::RunCmd;
use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, KillBlockchain, ExportState, DataFormat};
use presale::ImportWallet;
use account::{AccountCmd, NewAccount, ListAccounts, ImportAccounts, ImportFromGethAccounts};
use snapshot::{self, SnapshotCommand};

const AUTHCODE_FILENAME: &'static str = "authcodes";

#[derive(Debug, PartialEq)]
pub enum Cmd {
	Run(RunCmd),
	Version,
	Account(AccountCmd),
	ImportPresaleWallet(ImportWallet),
	Blockchain(BlockchainCmd),
	SignerToken(SignerConfiguration),
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
}

pub struct Execute {
	pub logger: LogConfig,
	pub cmd: Cmd,
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
	pub args: Args,
}

impl Configuration {
	pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
		let args = Args::parse(command)?;

		let config = Configuration {
			args: args,
		};

		Ok(config)
	}

	pub fn into_command(self) -> Result<Execute, String> {
		let dirs = self.directories();
		let pruning = self.args.flag_pruning.parse()?;
		let pruning_history = self.args.flag_pruning_history;
		let vm_type = self.vm_type()?;
		let mode = match self.args.flag_mode.as_ref() { "last" => None, mode => Some(to_mode(&mode, self.args.flag_mode_timeout, self.args.flag_mode_alarm)?), };
		let update_policy = self.update_policy()?;
		let logger_config = self.logger_config();
		let http_conf = self.http_config()?;
		let ipc_conf = self.ipc_config()?;
		let net_conf = self.net_config()?;
		let network_id = self.network_id();
		let cache_config = self.cache_config();
		let spec = self.chain().parse()?;
		let tracing = self.args.flag_tracing.parse()?;
		let fat_db = self.args.flag_fat_db.parse()?;
		let compaction = self.args.flag_db_compaction.parse()?;
		let wal = !self.args.flag_fast_and_loose;
		match self.args.flag_warp {
			// Logging is not initialized yet, so we print directly to stderr
			Some(true) if fat_db == Switch::On => writeln!(&mut stderr(), "Warning: Warp Sync is disabled because Fat DB is turned on").expect("Error writing to stderr"),
			Some(true) if tracing == Switch::On => writeln!(&mut stderr(), "Warning: Warp Sync is disabled because tracing is turned on").expect("Error writing to stderr"),
			Some(true) if pruning == Pruning::Specific(Algorithm::Archive) => writeln!(&mut stderr(), "Warning: Warp Sync is disabled because pruning mode is set to archive").expect("Error writing to stderr"),
			_ => {},
		};
		let warp_sync = !self.args.flag_no_warp && fat_db != Switch::On && tracing != Switch::On && pruning != Pruning::Specific(Algorithm::Archive);
		let geth_compatibility = self.args.flag_geth;
		let ui_address = self.ui_port().map(|port| (self.ui_interface(), port));
		let dapps_conf = self.dapps_config();
		let ipfs_conf = self.ipfs_config();
		let signer_conf = self.signer_config();
		let secretstore_conf = self.secretstore_config();
		let format = self.format()?;

		let cmd = if self.args.flag_version {
			Cmd::Version
		} else if self.args.cmd_signer {
			let mut authfile = PathBuf::from(signer_conf.signer_path.clone());
			authfile.push(AUTHCODE_FILENAME);

			if self.args.cmd_new_token {
				Cmd::SignerToken(signer_conf)
			} else if self.args.cmd_sign {
				let pwfile = self.args.flag_password.get(0).map(|pwfile| {
					PathBuf::from(pwfile)
				});
				Cmd::SignerSign {
					id: self.args.arg_id,
					pwfile: pwfile,
					port: signer_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_reject  {
				Cmd::SignerReject {
					id: self.args.arg_id,
					port: signer_conf.port,
					authfile: authfile,
				}
			} else if self.args.cmd_list  {
				Cmd::SignerList {
					port: signer_conf.port,
					authfile: authfile,
				}
			} else {
				unreachable!();
			}
		} else if self.args.cmd_tools && self.args.cmd_hash {
			Cmd::Hash(self.args.arg_file)
		} else if self.args.cmd_db && self.args.cmd_kill {
			Cmd::Blockchain(BlockchainCmd::Kill(KillBlockchain {
				spec: spec,
				dirs: dirs,
				pruning: pruning,
			}))
		} else if self.args.cmd_account {
			let account_cmd = if self.args.cmd_new {
				let new_acc = NewAccount {
					iterations: self.args.flag_keys_iterations,
					path: dirs.keys,
					spec: spec,
					password_file: self.args.flag_password.first().cloned(),
				};
				AccountCmd::New(new_acc)
			} else if self.args.cmd_list {
				let list_acc = ListAccounts {
					path: dirs.keys,
					spec: spec,
				};
				AccountCmd::List(list_acc)
			} else if self.args.cmd_import {
				let import_acc = ImportAccounts {
					from: self.args.arg_path.clone(),
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
				iterations: self.args.flag_keys_iterations,
				path: dirs.keys,
				spec: spec,
				wallet_path: self.args.arg_path.first().unwrap().clone(),
				password_file: self.args.flag_password.first().cloned(),
			};
			Cmd::ImportPresaleWallet(presale_cmd)
		} else if self.args.cmd_import {
			let import_cmd = ImportBlockchain {
				spec: spec,
				cache_config: cache_config,
				dirs: dirs,
				file_path: self.args.arg_file.clone(),
				format: format,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.flag_pruning_memory,
				compaction: compaction,
				wal: wal,
				tracing: tracing,
				fat_db: fat_db,
				vm_type: vm_type,
				check_seal: !self.args.flag_no_seal_check,
				with_color: logger_config.color,
				verifier_settings: self.verifier_settings(),
			};
			Cmd::Blockchain(BlockchainCmd::Import(import_cmd))
		} else if self.args.cmd_export {
			if self.args.cmd_blocks {
				let export_cmd = ExportBlockchain {
					spec: spec,
					cache_config: cache_config,
					dirs: dirs,
					file_path: self.args.arg_file.clone(),
					format: format,
					pruning: pruning,
					pruning_history: pruning_history,
					pruning_memory: self.args.flag_pruning_memory,
					compaction: compaction,
					wal: wal,
					tracing: tracing,
					fat_db: fat_db,
					from_block: to_block_id(&self.args.flag_from)?,
					to_block: to_block_id(&self.args.flag_to)?,
					check_seal: !self.args.flag_no_seal_check,
				};
				Cmd::Blockchain(BlockchainCmd::Export(export_cmd))
			} else if self.args.cmd_state {
				let export_cmd = ExportState {
					spec: spec,
					cache_config: cache_config,
					dirs: dirs,
					file_path: self.args.arg_file.clone(),
					format: format,
					pruning: pruning,
					pruning_history: pruning_history,
					pruning_memory: self.args.flag_pruning_memory,
					compaction: compaction,
					wal: wal,
					tracing: tracing,
					fat_db: fat_db,
					at: to_block_id(&self.args.flag_at)?,
					storage: !self.args.flag_no_storage,
					code: !self.args.flag_no_code,
					min_balance: self.args.flag_min_balance.and_then(|s| to_u256(&s).ok()),
					max_balance: self.args.flag_max_balance.and_then(|s| to_u256(&s).ok()),
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
				pruning_memory: self.args.flag_pruning_memory,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_file.clone(),
				wal: wal,
				kind: snapshot::Kind::Take,
				block_at: to_block_id(&self.args.flag_at)?,
			};
			Cmd::Snapshot(snapshot_cmd)
		} else if self.args.cmd_restore {
			let restore_cmd = SnapshotCommand {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.flag_pruning_memory,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_file.clone(),
				wal: wal,
				kind: snapshot::Kind::Restore,
				block_at: to_block_id("latest")?, // unimportant.
			};
			Cmd::Snapshot(restore_cmd)
		} else {
			let daemon = if self.args.cmd_daemon {
				Some(self.args.arg_pid_file.clone())
			} else {
				None
			};

			let verifier_settings = self.verifier_settings();

			// Special presets are present for the dev chain.
			let (gas_pricer, miner_options) = match spec {
				SpecType::Dev => (GasPricerConfig::Fixed(0.into()), self.miner_options(0)?),
				_ => (self.gas_pricer_config()?, self.miner_options(self.args.flag_reseal_min_period)?),
			};

			let run_cmd = RunCmd {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				pruning_memory: self.args.flag_pruning_memory,
				daemon: daemon,
				logger_config: logger_config.clone(),
				miner_options: miner_options,
				http_conf: http_conf,
				ipc_conf: ipc_conf,
				net_conf: net_conf,
				network_id: network_id,
				acc_conf: self.accounts_config()?,
				gas_pricer: gas_pricer,
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
				geth_compatibility: geth_compatibility,
				ui_address: ui_address,
				net_settings: self.network_settings(),
				dapps_conf: dapps_conf,
				ipfs_conf: ipfs_conf,
				signer_conf: signer_conf,
				secretstore_conf: secretstore_conf,
				dapp: self.dapp_to_open()?,
				ui: self.args.cmd_ui,
				name: self.args.flag_identity,
				custom_bootnodes: self.args.flag_bootnodes.is_some(),
				no_periodic_snapshot: self.args.flag_no_periodic_snapshot,
				check_seal: !self.args.flag_no_seal_check,
				download_old_blocks: !self.args.flag_no_ancient_blocks,
				verifier_settings: verifier_settings,
			};
			Cmd::Run(run_cmd)
		};

		Ok(Execute {
			logger: logger_config,
			cmd: cmd,
		})
	}

	fn vm_type(&self) -> Result<VMType, String> {
		if self.args.flag_jitvm {
			VMType::jit().ok_or("Parity is built without the JIT EVM.".into())
		} else {
			Ok(VMType::Interpreter)
		}
	}

	fn miner_extras(&self) -> Result<MinerExtras, String> {
		let extras = MinerExtras {
			author: self.author()?,
			extra_data: self.extra_data()?,
			gas_floor_target: to_u256(&self.args.flag_gas_floor_target)?,
			gas_ceil_target: to_u256(&self.args.flag_gas_cap)?,
			transactions_limit: self.args.flag_tx_queue_size,
			engine_signer: self.engine_signer()?,
		};

		Ok(extras)
	}

	fn author(&self) -> Result<Address, String> {
		to_address(self.args.flag_etherbase.clone().or(self.args.flag_author.clone()))
	}

	fn engine_signer(&self) -> Result<Address, String> {
		to_address(self.args.flag_engine_signer.clone())
	}

	fn format(&self) -> Result<Option<DataFormat>, String> {
		match self.args.flag_format {
			Some(ref f) => Ok(Some(f.parse()?)),
			None => Ok(None),
		}
	}

	fn cache_config(&self) -> CacheConfig {
		match self.args.flag_cache_size.or(self.args.flag_cache) {
			Some(size) => CacheConfig::new_with_total_cache_size(size),
			None => CacheConfig::new(
				self.args.flag_cache_size_db,
				self.args.flag_cache_size_blocks,
				self.args.flag_cache_size_queue,
				self.args.flag_cache_size_state,
			),
		}
	}

	fn logger_config(&self) -> LogConfig {
		LogConfig {
			mode: self.args.flag_logging.clone(),
			color: !self.args.flag_no_color && !cfg!(windows),
			file: self.args.flag_log_file.clone(),
		}
	}

	fn chain(&self) -> String {
		if self.args.flag_testnet {
			"testnet".to_owned()
		} else {
			self.args.flag_chain.clone()
		}
	}

	fn max_peers(&self) -> u32 {
		let peers = self.args.flag_max_peers as u32;
		max(self.min_peers(), peers)
	}

	fn allow_ips(&self) -> Result<AllowIP, String> {
		match self.args.flag_allow_ips.as_str() {
			"all" => Ok(AllowIP::All),
			"public" => Ok(AllowIP::Public),
			"private" => Ok(AllowIP::Private),
			_ => Err("Invalid IP filter value".to_owned()),
		}
	}

	fn min_peers(&self) -> u32 {
		self.args.flag_peers.unwrap_or(self.args.flag_min_peers) as u32
	}

	fn max_pending_peers(&self) -> u32 {
		self.args.flag_max_pending_peers as u32
	}

	fn snapshot_peers(&self) -> u32 {
		self.args.flag_snapshot_peers as u32
	}

	fn work_notify(&self) -> Vec<String> {
		self.args.flag_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	fn accounts_config(&self) -> Result<AccountsConfig, String> {
		let cfg = AccountsConfig {
			iterations: self.args.flag_keys_iterations,
			testnet: self.args.flag_testnet,
			password_files: self.args.flag_password.clone(),
			unlocked_accounts: to_addresses(&self.args.flag_unlock)?,
			enable_hardware_wallets: !self.args.flag_no_hardware_wallets,
		};

		Ok(cfg)
	}

	fn stratum_options(&self) -> Result<Option<StratumOptions>, String> {
		if self.args.flag_stratum {
			Ok(Some(StratumOptions {
				io_path: self.directories().db,
				listen_addr: self.stratum_interface(),
				port: self.args.flag_stratum_port,
				secret: self.args.flag_stratum_secret.as_ref().map(|s| s.parse::<H256>().unwrap_or_else(|_| s.sha3())),
			}))
		} else { Ok(None) }
	}

	fn miner_options(&self, reseal_min_period: u64) -> Result<MinerOptions, String> {
		let reseal = self.args.flag_reseal_on_txs.parse::<ResealPolicy>()?;

		let options = MinerOptions {
			new_work_notify: self.work_notify(),
			force_sealing: self.args.flag_force_sealing,
			reseal_on_external_tx: reseal.external,
			reseal_on_own_tx: reseal.own,
			tx_gas_limit: match self.args.flag_tx_gas_limit {
				Some(ref d) => to_u256(d)?,
				None => U256::max_value(),
			},
			tx_queue_size: self.args.flag_tx_queue_size,
			tx_queue_gas_limit: to_gas_limit(&self.args.flag_tx_queue_gas)?,
			tx_queue_strategy: to_queue_strategy(&self.args.flag_tx_queue_strategy)?,
			pending_set: to_pending_set(&self.args.flag_relay_set)?,
			reseal_min_period: Duration::from_millis(reseal_min_period),
			work_queue_size: self.args.flag_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
			tx_queue_banning: match self.args.flag_tx_time_limit {
				Some(limit) => Banning::Enabled {
					min_offends: self.args.flag_tx_queue_ban_count,
					offend_threshold: Duration::from_millis(limit),
					ban_duration: Duration::from_secs(self.args.flag_tx_queue_ban_time as u64),
				},
				None => Banning::Disabled,
			},
			refuse_service_transactions: self.args.flag_refuse_service_transactions,
		};

		Ok(options)
	}

	fn signer_config(&self) -> SignerConfiguration {
		SignerConfiguration {
			enabled: self.ui_enabled(),
			port: self.args.flag_ui_port,
			interface: self.ui_interface(),
			signer_path: self.directories().signer,
			skip_origin_validation: self.args.flag_ui_no_validation,
		}
	}

	fn dapps_config(&self) -> DappsConfiguration {
		DappsConfiguration {
			enabled: self.dapps_enabled(),
			interface: self.dapps_interface(),
			port: self.args.flag_dapps_port,
			hosts: self.dapps_hosts(),
			cors: self.dapps_cors(),
			user: self.args.flag_dapps_user.clone(),
			pass: self.args.flag_dapps_pass.clone(),
			dapps_path: PathBuf::from(self.directories().dapps),
			extra_dapps: if self.args.cmd_dapp {
				self.args.arg_path.iter().map(|path| PathBuf::from(path)).collect()
			} else {
				vec![]
			},
			all_apis: self.args.flag_dapps_apis_all,
		}
	}

	fn secretstore_config(&self) -> SecretStoreConfiguration {
		SecretStoreConfiguration {
			enabled: self.secretstore_enabled(),
			interface: self.secretstore_interface(),
			port: self.args.flag_secretstore_port,
			data_path: self.directories().secretstore,
		}
	}

	fn ipfs_config(&self) -> IpfsConfiguration {
		IpfsConfiguration {
			enabled: self.args.flag_ipfs_api,
			port: self.args.flag_ipfs_api_port,
			interface: self.ipfs_interface(),
			cors: self.ipfs_cors(),
			hosts: self.ipfs_hosts(),
		}
	}

	fn dapp_to_open(&self) -> Result<Option<String>, String> {
		if !self.args.cmd_dapp {
			return Ok(None);
		}
		let path = self.args.arg_path.get(0).map(String::as_str).unwrap_or(".");
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

		if let Some(d) = self.args.flag_gasprice.as_ref() {
			return Ok(GasPricerConfig::Fixed(to_u256(d)?));
		}

		let usd_per_tx = to_price(&self.args.flag_usd_per_tx)?;
		if "auto" == self.args.flag_usd_per_eth.as_str() {
			// Just a very rough estimate to avoid accepting
			// ZGP transactions before the price is fetched
			// if user does not want it.
			let last_known_usd_per_eth = 10.0;
			return Ok(GasPricerConfig::Calibrated {
				initial_minimum: wei_per_gas(usd_per_tx, last_known_usd_per_eth),
				usd_per_tx: usd_per_tx,
				recalibration_period: to_duration(self.args.flag_price_update_period.as_str())?,
			});
		}

		let usd_per_eth = to_price(&self.args.flag_usd_per_eth)?;
		let wei_per_gas = wei_per_gas(usd_per_tx, usd_per_eth);

		info!(
			"Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)",
			Colour::White.bold().paint(format!("US${}", usd_per_eth)),
			Colour::Yellow.bold().paint(format!("{}", wei_per_gas))
		);

		Ok(GasPricerConfig::Fixed(wei_per_gas))
	}

	fn extra_data(&self) -> Result<Bytes, String> {
		match self.args.flag_extradata.as_ref().or(self.args.flag_extra_data.as_ref()) {
			Some(x) if x.len() <= 32 => Ok(x.as_bytes().to_owned()),
			None => Ok(version_data()),
			Some(_) => Err("Extra data must be at most 32 characters".into()),
		}
	}

	fn init_reserved_nodes(&self) -> Result<Vec<String>, String> {
		use std::fs::File;

		match self.args.flag_reserved_peers {
			Some(ref path) => {
				let mut buffer = String::new();
				let mut node_file = File::open(path).map_err(|e| format!("Error opening reserved nodes file: {}", e))?;
				node_file.read_to_string(&mut buffer).map_err(|_| "Error reading reserved node file")?;
				let lines = buffer.lines().map(|s| s.trim().to_owned()).filter(|s| !s.is_empty()).collect::<Vec<_>>();
				if let Some(invalid) = lines.iter().find(|s| !is_valid_node_url(s)) {
					return Err(format!("Invalid node address format given for a boot node: {}", invalid));
				}
				Ok(lines)
			},
			None => Ok(Vec::new())
		}
	}

	fn net_addresses(&self) -> Result<(Option<SocketAddr>, Option<SocketAddr>), String> {
		let port = self.args.flag_port;
		let listen_address = Some(SocketAddr::new("0.0.0.0".parse().unwrap(), port));
		let public_address = if self.args.flag_nat.starts_with("extip:") {
			let host = &self.args.flag_nat[6..];
			let host = host.parse().map_err(|_| format!("Invalid host given with `--nat extip:{}`", host))?;
			Some(SocketAddr::new(host, port))
		} else {
			None
		};
		Ok((listen_address, public_address))
	}

	fn net_config(&self) -> Result<NetworkConfiguration, String> {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = to_bootnodes(&self.args.flag_bootnodes)?;
		let (listen, public) = self.net_addresses()?;
		ret.listen_address = listen.map(|l| format!("{}", l));
		ret.public_address = public.map(|p| format!("{}", p));
		ret.use_secret = match self.args.flag_node_key.as_ref()
			.map(|s| s.parse::<Secret>().or_else(|_| Secret::from_slice(&s.sha3())).map_err(|e| format!("Invalid key: {:?}", e))
			) {
			None => None,
			Some(Ok(key)) => Some(key),
			Some(Err(err)) => return Err(err),
		};
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.max_peers = self.max_peers();
		ret.min_peers = self.min_peers();
		ret.snapshot_peers = self.snapshot_peers();
		ret.allow_ips = self.allow_ips()?;
		ret.max_pending_peers = self.max_pending_peers();
		let mut net_path = PathBuf::from(self.directories().base);
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret.reserved_nodes = self.init_reserved_nodes()?;
		ret.allow_non_reserved = !self.args.flag_reserved_only;
		Ok(ret)
	}

	fn network_id(&self) -> Option<u64> {
		self.args.flag_network_id.or(self.args.flag_networkid)
	}

	fn rpc_apis(&self) -> String {
		let mut apis = self.args.flag_rpcapi.clone().unwrap_or(self.args.flag_jsonrpc_apis.clone());
		if self.args.flag_geth {
			if !apis.is_empty() {
				apis.push_str(",");
			}
			apis.push_str("personal");
		}
		apis
	}

	fn cors(cors: Option<&String>) -> Option<Vec<String>> {
		cors.map(|ref c| c.split(',').map(Into::into).collect())
	}

	fn rpc_cors(&self) -> Option<Vec<String>> {
		let cors = self.args.flag_jsonrpc_cors.as_ref().or(self.args.flag_rpccorsdomain.as_ref());
		Self::cors(cors)
	}

	fn ipfs_cors(&self) -> Option<Vec<String>> {
		Self::cors(self.args.flag_ipfs_api_cors.as_ref())
	}

	fn dapps_cors(&self) -> Option<Vec<String>> {
		Self::cors(self.args.flag_dapps_cors.as_ref())
	}

	fn hosts(hosts: &str) -> Option<Vec<String>> {
		match hosts {
			"none" => return Some(Vec::new()),
			"all" => return None,
			_ => {}
		}
		let hosts = hosts.split(',').map(Into::into).collect();
		Some(hosts)
	}

	fn rpc_hosts(&self) -> Option<Vec<String>> {
		Self::hosts(&self.args.flag_jsonrpc_hosts)
	}

	fn dapps_hosts(&self) -> Option<Vec<String>> {
		Self::hosts(&self.args.flag_dapps_hosts)
	}

	fn ipfs_hosts(&self) -> Option<Vec<String>> {
		Self::hosts(&self.args.flag_ipfs_api_hosts)
	}

	fn ipc_config(&self) -> Result<IpcConfiguration, String> {
		let conf = IpcConfiguration {
			enabled: !(self.args.flag_ipcdisable || self.args.flag_ipc_off || self.args.flag_no_ipc),
			socket_addr: self.ipc_path(),
			apis: {
				let mut apis = self.args.flag_ipcapi.clone().unwrap_or(self.args.flag_ipc_apis.clone());
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
			enabled: !self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc,
			interface: self.rpc_interface(),
			port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
			apis: self.rpc_apis().parse()?,
			hosts: self.rpc_hosts(),
			cors: self.rpc_cors(),
		};

		Ok(conf)
	}

	fn network_settings(&self) -> NetworkSettings {
		NetworkSettings {
			name: self.args.flag_identity.clone(),
			chain: self.chain(),
			network_port: self.args.flag_port,
			rpc_enabled: !self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc,
			rpc_interface: self.args.flag_rpcaddr.clone().unwrap_or(self.args.flag_jsonrpc_interface.clone()),
			rpc_port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
		}
	}

	fn update_policy(&self) -> Result<UpdatePolicy, String> {
		Ok(UpdatePolicy {
			enable_downloading: !self.args.flag_no_download,
			require_consensus: !self.args.flag_no_consensus,
			filter: match self.args.flag_auto_update.as_ref() {
				"none" => UpdateFilter::None,
				"critical" => UpdateFilter::Critical,
				"all" => UpdateFilter::All,
				_ => return Err("Invalid value for `--auto-update`. See `--help` for more information.".into()),
			},
			track: match self.args.flag_release_track.as_ref() {
				"stable" => ReleaseTrack::Stable,
				"beta" => ReleaseTrack::Beta,
				"nightly" => ReleaseTrack::Nightly,
				"testing" => ReleaseTrack::Testing,
				"current" => ReleaseTrack::Unknown,
				_ => return Err("Invalid value for `--releases-track`. See `--help` for more information.".into()),
			},
			path: default_hypervisor_path(),
		})
	}

	fn directories(&self) -> Directories {
		use util::path;

		let local_path = default_local_path();
		let base_path = self.args.flag_base_path.as_ref().map_or_else(|| default_data_path(), |s| s.clone());
		let data_path = replace_home("", self.args.flag_datadir.as_ref().unwrap_or(&base_path));
		let base_db_path = if self.args.flag_base_path.is_some() && self.args.flag_db_path.is_none() {
			// If base_path is set and db_path is not we default to base path subdir instead of LOCAL.
			"$BASE/chains"
		} else {
			self.args.flag_db_path.as_ref().map_or(dir::CHAINS_PATH, |s| &s)
		};

		let db_path = replace_home_for_db(&data_path, &local_path, &base_db_path);
		let keys_path = replace_home(&data_path, &self.args.flag_keys_path);
		let dapps_path = replace_home(&data_path, &self.args.flag_dapps_path);
		let secretstore_path = replace_home(&data_path, &self.args.flag_secretstore_path);
		let ui_path = replace_home(&data_path, &self.args.flag_ui_path);

		if self.args.flag_geth && !cfg!(windows) {
			let geth_root  = if self.chain() == "testnet".to_owned() { path::ethereum::test() } else {  path::ethereum::default() };
			::std::fs::create_dir_all(geth_root.as_path()).unwrap_or_else(
				|e| warn!("Failed to create '{}' for geth mode: {}", &geth_root.to_str().unwrap(), e));
		}

		if cfg!(feature = "ipc") && !cfg!(feature = "windows") {
			let mut path_buf = PathBuf::from(data_path.clone());
			path_buf.push("ipc");
			let ipc_path = path_buf.to_str().unwrap();
			::std::fs::create_dir_all(ipc_path).unwrap_or_else(
				|e| warn!("Failed to directory '{}' for ipc sockets: {}", ipc_path, e)
			);
		}

		Directories {
			keys: keys_path,
			base: data_path,
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
			parity_ipc_path(&self.directories().base, &self.args.flag_ipcpath.clone().unwrap_or(self.args.flag_ipc_path.clone()))
		}
	}

	fn ui_port(&self) -> Option<u16> {
		if !self.ui_enabled() {
			None
		} else {
			Some(self.args.flag_ui_port)
		}
	}

	fn ui_interface(&self) -> String {
		match self.args.flag_ui_interface.as_str() {
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn interface(interface: &str) -> String {
		match interface {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn rpc_interface(&self) -> String {
		Self::interface(&self.network_settings().rpc_interface)
	}

	fn dapps_interface(&self) -> String {
		match self.args.flag_dapps_interface.as_str() {
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn ipfs_interface(&self) -> String {
		Self::interface(&self.args.flag_ipfs_api_interface)
	}

	fn secretstore_interface(&self) -> String {
		match self.args.flag_secretstore_interface.as_str() {
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn stratum_interface(&self) -> String {
		Self::interface(&self.args.flag_stratum_interface)
	}

	fn dapps_enabled(&self) -> bool {
		!self.args.flag_dapps_off && !self.args.flag_no_dapps && cfg!(feature = "dapps")
	}

	fn secretstore_enabled(&self) -> bool {
		!self.args.flag_no_secretstore && cfg!(feature = "secretstore")
	}

	fn ui_enabled(&self) -> bool {
		if self.args.flag_force_ui {
			return true;
		}

		let ui_disabled = self.args.flag_unlock.is_some() ||
			self.args.flag_geth ||
			self.args.flag_no_ui;

		!ui_disabled
	}

	fn verifier_settings(&self) -> VerifierSettings {
		let mut settings = VerifierSettings::default();
		settings.scale_verifiers = self.args.flag_scale_verifiers;
		if let Some(num_verifiers) = self.args.flag_num_verifiers {
			settings.num_verifiers = num_verifiers;
		}

		settings
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use cli::Args;
	use ethcore_rpc::NetworkSettings;
	use ethcore::client::{VMType, BlockId};
	use ethcore::miner::{MinerOptions, PrioritizationStrategy};
	use helpers::{default_network_config};
	use run::RunCmd;
	use dir::{Directories, default_hypervisor_path};
	use signer::{Configuration as SignerConfiguration};
	use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, DataFormat, ExportState};
	use presale::ImportWallet;
	use params::SpecType;
	use account::{AccountCmd, NewAccount, ImportAccounts, ListAccounts};
	use devtools::{RandomTempPath};
	use updater::{UpdatePolicy, UpdateFilter, ReleaseTrack};
	use std::io::Write;
	use std::fs::{File, create_dir};

	#[derive(Debug, PartialEq)]
	struct TestPasswordReader(&'static str);

	fn parse(args: &[&str]) -> Configuration {
		Configuration {
			args: Args::parse_without_config(args).unwrap(),
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
			pruning_memory: 75,
			compaction: Default::default(),
			wal: true,
			tracing: Default::default(),
			fat_db: Default::default(),
			vm_type: VMType::Interpreter,
			check_seal: true,
			with_color: !cfg!(windows),
			verifier_settings: Default::default(),
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
			pruning_memory: 75,
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
			pruning_memory: 75,
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
			pruning_memory: 75,
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
		assert_eq!(conf.into_command().unwrap().cmd, Cmd::SignerToken(SignerConfiguration {
			enabled: true,
			signer_path: expected,
			interface: "127.0.0.1".into(),
			port: 8180,
			skip_origin_validation: false,
		}));
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
			pruning_memory: 75,
			daemon: None,
			logger_config: Default::default(),
			miner_options: Default::default(),
			http_conf: Default::default(),
			ipc_conf: Default::default(),
			net_conf: default_network_config(),
			network_id: None,
			warp_sync: true,
			acc_conf: Default::default(),
			gas_pricer: Default::default(),
			miner_extras: Default::default(),
			update_policy: UpdatePolicy { enable_downloading: true, require_consensus: true, filter: UpdateFilter::Critical, track: ReleaseTrack::Unknown, path: default_hypervisor_path() },
			mode: Default::default(),
			tracing: Default::default(),
			compaction: Default::default(),
			wal: true,
			vm_type: Default::default(),
			geth_compatibility: false,
			ui_address: Some(("127.0.0.1".into(), 8180)),
			net_settings: Default::default(),
			dapps_conf: Default::default(),
			ipfs_conf: Default::default(),
			signer_conf: Default::default(),
			secretstore_conf: Default::default(),
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
		};
		expected.secretstore_conf.enabled = cfg!(feature = "secretstore");
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
		let min_period = conf0.args.flag_reseal_min_period;
		assert_eq!(conf0.miner_options(min_period).unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasFactorAndGasPrice;
		assert_eq!(conf1.miner_options(min_period).unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasPriceOnly;
		assert_eq!(conf2.miner_options(min_period).unwrap(), mining_options);
		mining_options.tx_queue_strategy = PrioritizationStrategy::GasAndGasPrice;
		assert_eq!(conf3.miner_options(min_period).unwrap(), mining_options);
	}

	#[test]
	fn should_parse_updater_options() {
		// when
		let conf0 = parse(&["parity", "--release-track=testing"]);
		let conf1 = parse(&["parity", "--auto-update", "all", "--no-consensus"]);
		let conf2 = parse(&["parity", "--no-download", "--auto-update=all", "--release-track=beta"]);
		let conf3 = parse(&["parity", "--auto-update=xxx"]);

		// then
		assert_eq!(conf0.update_policy().unwrap(), UpdatePolicy{enable_downloading: true, require_consensus: true, filter: UpdateFilter::Critical, track: ReleaseTrack::Testing, path: default_hypervisor_path()});
		assert_eq!(conf1.update_policy().unwrap(), UpdatePolicy{enable_downloading: true, require_consensus: false, filter: UpdateFilter::All, track: ReleaseTrack::Unknown, path: default_hypervisor_path()});
		assert_eq!(conf2.update_policy().unwrap(), UpdatePolicy{enable_downloading: false, require_consensus: true, filter: UpdateFilter::All, track: ReleaseTrack::Beta, path: default_hypervisor_path()});
		assert!(conf3.update_policy().is_err());
	}

	#[test]
	fn should_parse_network_settings() {
		// given

		// when
		let conf = parse(&["parity", "--testnet", "--identity", "testname"]);

		// then
		assert_eq!(conf.network_settings(), NetworkSettings {
			name: "testname".to_owned(),
			chain: "testnet".to_owned(),
			network_port: 30303,
			rpc_enabled: true,
			rpc_interface: "local".to_owned(),
			rpc_port: 8545,
		});
	}

	#[test]
	fn should_parse_rpc_settings_with_geth_compatiblity() {
		// given
		fn assert(conf: Configuration) {
			let net = conf.network_settings();
			assert_eq!(net.rpc_enabled, true);
			assert_eq!(net.rpc_interface, "all".to_owned());
			assert_eq!(net.rpc_port, 8000);
			assert_eq!(conf.rpc_cors(), Some(vec!["*".to_owned()]));
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
		let conf3 = parse(&["parity", "--jsonrpc-hosts", "ethcore.io,something.io"]);

		// then
		assert_eq!(conf0.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf1.rpc_hosts(), Some(Vec::new()));
		assert_eq!(conf2.rpc_hosts(), None);
		assert_eq!(conf3.rpc_hosts(), Some(vec!["ethcore.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_parse_dapps_hosts() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--dapps-hosts", "none"]);
		let conf2 = parse(&["parity", "--dapps-hosts", "all"]);
		let conf3 = parse(&["parity", "--dapps-hosts", "ethcore.io,something.io"]);

		// then
		assert_eq!(conf0.dapps_hosts(), Some(Vec::new()));
		assert_eq!(conf1.dapps_hosts(), Some(Vec::new()));
		assert_eq!(conf2.dapps_hosts(), None);
		assert_eq!(conf3.dapps_hosts(), Some(vec!["ethcore.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_parse_ipfs_hosts() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--ipfs-api-hosts", "none"]);
		let conf2 = parse(&["parity", "--ipfs-api-hosts", "all"]);
		let conf3 = parse(&["parity", "--ipfs-api-hosts", "ethcore.io,something.io"]);

		// then
		assert_eq!(conf0.ipfs_hosts(), Some(Vec::new()));
		assert_eq!(conf1.ipfs_hosts(), Some(Vec::new()));
		assert_eq!(conf2.ipfs_hosts(), None);
		assert_eq!(conf3.ipfs_hosts(), Some(vec!["ethcore.io".into(), "something.io".into()]));
	}

	#[test]
	fn should_parse_ipfs_cors() {
		// given

		// when
		let conf0 = parse(&["parity"]);
		let conf1 = parse(&["parity", "--ipfs-api-cors", "*"]);
		let conf2 = parse(&["parity", "--ipfs-api-cors", "http://ethcore.io,http://something.io"]);

		// then
		assert_eq!(conf0.ipfs_cors(), None);
		assert_eq!(conf1.ipfs_cors(), Some(vec!["*".into()]));
		assert_eq!(conf2.ipfs_cors(), Some(vec!["http://ethcore.io".into(),"http://something.io".into()]));
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
	fn should_parse_signer_configration() {
		// given

		// when
		let conf0 = parse(&["parity", "--ui-path", "signer"]);
		let conf1 = parse(&["parity", "--ui-path", "signer", "--ui-no-validation"]);
		let conf2 = parse(&["parity", "--ui-path", "signer", "--ui-port", "3123"]);
		let conf3 = parse(&["parity", "--ui-path", "signer", "--ui-interface", "test"]);

		// then
		assert_eq!(conf0.signer_config(), SignerConfiguration {
			enabled: true,
			port: 8180,
			interface: "127.0.0.1".into(),
			signer_path: "signer".into(),
			skip_origin_validation: false,
		});
		assert_eq!(conf1.signer_config(), SignerConfiguration {
			enabled: true,
			port: 8180,
			interface: "127.0.0.1".into(),
			signer_path: "signer".into(),
			skip_origin_validation: true,
		});
		assert_eq!(conf2.signer_config(), SignerConfiguration {
			enabled: true,
			port: 3123,
			interface: "127.0.0.1".into(),
			signer_path: "signer".into(),
			skip_origin_validation: false,
		});
		assert_eq!(conf3.signer_config(), SignerConfiguration {
			enabled: true,
			port: 8180,
			interface: "test".into(),
			signer_path: "signer".into(),
			skip_origin_validation: false,
		});
	}

	#[test]
	fn should_parse_dapp_opening() {
		// given
		let temp = RandomTempPath::new();
		let name = temp.file_name().unwrap().to_str().unwrap();
		create_dir(temp.as_str().to_owned()).unwrap();

		// when
		let conf0 = parse(&["parity", "dapp", temp.to_str().unwrap()]);

		// then
		assert_eq!(conf0.dapp_to_open(), Ok(Some(name.into())));
		let extra_dapps = conf0.dapps_config().extra_dapps;
		assert_eq!(extra_dapps, vec![temp.to_owned()]);
	}

	#[test]
	fn should_not_bail_on_empty_line_in_reserved_peers() {
		let temp = RandomTempPath::new();
		create_dir(temp.as_str().to_owned()).unwrap();
		let filename = temp.as_str().to_owned() + "/peers";
		File::create(filename.clone()).unwrap().write_all(b"  \n\t\n").unwrap();
		let args = vec!["parity", "--reserved-peers", &filename];
		let conf = Configuration::parse(&args).unwrap();
		assert!(conf.init_reserved_nodes().is_ok());
	}

	#[test]
	fn test_dev_chain() {
		let args = vec!["parity", "--chain", "dev"];
		let conf = parse(&args);
		match conf.into_command().unwrap().cmd {
			Cmd::Run(c) => {
				assert_eq!(c.gas_pricer, GasPricerConfig::Fixed(0.into()));
				assert_eq!(c.miner_options.reseal_min_period, Duration::from_millis(0));
			},
			_ => panic!("Should be Cmd::Run"),
		}
	}
}
