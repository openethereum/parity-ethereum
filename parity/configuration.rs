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

use std::time::Duration;
use std::io::Read;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::cmp::max;
use cli::{Args, ArgsError};
use util::{Hashable, U256, Uint, Bytes, version_data, Secret, Address};
use util::log::Colour;
use ethsync::{NetworkConfiguration, is_valid_node_url};
use ethcore::client::{VMType, Mode};
use ethcore::miner::MinerOptions;

use rpc::{IpcConfiguration, HttpConfiguration};
use ethcore_rpc::NetworkSettings;
use cache::CacheConfig;
use helpers::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_price, replace_home,
geth_ipc_path, parity_ipc_path, to_bootnodes, to_addresses, to_address, to_gas_limit, to_queue_strategy};
use params::{ResealPolicy, AccountsConfig, GasPricerConfig, MinerExtras};
use ethcore_logger::Config as LogConfig;
use dir::Directories;
use dapps::Configuration as DappsConfiguration;
use signer::Configuration as SignerConfiguration;
use run::RunCmd;
use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, DataFormat};
use presale::ImportWallet;
use account::{AccountCmd, NewAccount, ImportAccounts, ImportFromGethAccounts};
use snapshot::{self, SnapshotCommand};

#[derive(Debug, PartialEq)]
pub enum Cmd {
	Run(RunCmd),
	Version,
	Account(AccountCmd),
	ImportPresaleWallet(ImportWallet),
	Blockchain(BlockchainCmd),
	SignerToken(String),
	Snapshot(SnapshotCommand),
	Hash(Option<String>),
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
	pub args: Args,
}

impl Configuration {
	pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
		let args = try!(Args::parse(command));

		let config = Configuration {
			args: args,
		};

		Ok(config)
	}

	pub fn into_command(self) -> Result<Cmd, String> {
		let dirs = self.directories();
		let pruning = try!(self.args.flag_pruning.parse());
		let pruning_history = self.args.flag_pruning_history;
		let vm_type = try!(self.vm_type());
		let mode = try!(to_mode(&self.args.flag_mode, self.args.flag_mode_timeout, self.args.flag_mode_alarm));
		let miner_options = try!(self.miner_options());
		let logger_config = self.logger_config();
		let http_conf = try!(self.http_config());
		let ipc_conf = try!(self.ipc_config());
		let net_conf = try!(self.net_config());
		let network_id = try!(self.network_id());
		let cache_config = self.cache_config();
		let spec = try!(self.chain().parse());
		let tracing = try!(self.args.flag_tracing.parse());
		let fat_db = try!(self.args.flag_fat_db.parse());
		let compaction = try!(self.args.flag_db_compaction.parse());
		let wal = !self.args.flag_fast_and_loose;
		let enable_network = self.enable_network(&mode);
		let warp_sync = self.args.flag_warp;
		let geth_compatibility = self.args.flag_geth;
		let signer_port = self.signer_port();
		let dapps_conf = self.dapps_config();
		let signer_conf = self.signer_config();
		let format = try!(self.format());

		let cmd = if self.args.flag_version {
			Cmd::Version
		} else if self.args.cmd_signer && self.args.cmd_new_token {
			Cmd::SignerToken(dirs.signer)
		} else if self.args.cmd_tools && self.args.cmd_hash {
			Cmd::Hash(self.args.arg_file)
		} else if self.args.cmd_account {
			let account_cmd = if self.args.cmd_new {
				let new_acc = NewAccount {
					iterations: self.args.flag_keys_iterations,
					path: dirs.keys,
					password_file: self.args.flag_password.first().cloned(),
				};
				AccountCmd::New(new_acc)
			} else if self.args.cmd_list {
				AccountCmd::List(dirs.keys)
			} else if self.args.cmd_import {
				let import_acc = ImportAccounts {
					from: self.args.arg_path.clone(),
					to: dirs.keys,
				};
				AccountCmd::Import(import_acc)
			} else {
				unreachable!();
			};
			Cmd::Account(account_cmd)
		} else if self.args.flag_import_geth_keys {
        	let account_cmd = AccountCmd::ImportFromGeth(
				ImportFromGethAccounts {
					to: dirs.keys,
					testnet: self.args.flag_testnet
				}
			);
			Cmd::Account(account_cmd)
		} else if self.args.cmd_wallet {
			let presale_cmd = ImportWallet {
				iterations: self.args.flag_keys_iterations,
				path: dirs.keys,
				wallet_path: self.args.arg_path.first().unwrap().clone(),
				password_file: self.args.flag_password.first().cloned(),
			};
			Cmd::ImportPresaleWallet(presale_cmd)
		} else if self.args.cmd_import {
			let import_cmd = ImportBlockchain {
				spec: spec,
				logger_config: logger_config,
				cache_config: cache_config,
				dirs: dirs,
				file_path: self.args.arg_file.clone(),
				format: format,
				pruning: pruning,
				pruning_history: pruning_history,
				compaction: compaction,
				wal: wal,
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				vm_type: vm_type,
			};
			Cmd::Blockchain(BlockchainCmd::Import(import_cmd))
		} else if self.args.cmd_export {
			let export_cmd = ExportBlockchain {
				spec: spec,
				logger_config: logger_config,
				cache_config: cache_config,
				dirs: dirs,
				file_path: self.args.arg_file.clone(),
				format: format,
				pruning: pruning,
				pruning_history: pruning_history,
				compaction: compaction,
				wal: wal,
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				from_block: try!(to_block_id(&self.args.flag_from)),
				to_block: try!(to_block_id(&self.args.flag_to)),
			};
			Cmd::Blockchain(BlockchainCmd::Export(export_cmd))
		} else if self.args.cmd_snapshot {
			let snapshot_cmd = SnapshotCommand {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				logger_config: logger_config,
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_file.clone(),
				wal: wal,
				kind: snapshot::Kind::Take,
				block_at: try!(to_block_id(&self.args.flag_at)),
			};
			Cmd::Snapshot(snapshot_cmd)
		} else if self.args.cmd_restore {
			let restore_cmd = SnapshotCommand {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				logger_config: logger_config,
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				file_path: self.args.arg_file.clone(),
				wal: wal,
				kind: snapshot::Kind::Restore,
				block_at: try!(to_block_id("latest")), // unimportant.
			};
			Cmd::Snapshot(restore_cmd)
		} else {
			let daemon = if self.args.cmd_daemon {
				Some(self.args.arg_pid_file.clone())
			} else {
				None
			};

			let run_cmd = RunCmd {
				cache_config: cache_config,
				dirs: dirs,
				spec: spec,
				pruning: pruning,
				pruning_history: pruning_history,
				daemon: daemon,
				logger_config: logger_config,
				miner_options: miner_options,
				http_conf: http_conf,
				ipc_conf: ipc_conf,
				net_conf: net_conf,
				network_id: network_id,
				acc_conf: try!(self.accounts_config()),
				gas_pricer: try!(self.gas_pricer_config()),
				miner_extras: try!(self.miner_extras()),
				mode: mode,
				tracing: tracing,
				fat_db: fat_db,
				compaction: compaction,
				wal: wal,
				vm_type: vm_type,
				enable_network: enable_network,
				warp_sync: warp_sync,
				geth_compatibility: geth_compatibility,
				signer_port: signer_port,
				net_settings: self.network_settings(),
				dapps_conf: dapps_conf,
				signer_conf: signer_conf,
				ui: self.args.cmd_ui,
				name: self.args.flag_identity,
				custom_bootnodes: self.args.flag_bootnodes.is_some(),
				no_periodic_snapshot: self.args.flag_no_periodic_snapshot,
			};
			Cmd::Run(run_cmd)
		};

		Ok(cmd)
	}

	fn enable_network(&self, mode: &Mode) -> bool {
		match *mode {
			Mode::Dark(_) => false,
			_ => !self.args.flag_no_network,
		}
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
			author: try!(self.author()),
			extra_data: try!(self.extra_data()),
			gas_floor_target: try!(to_u256(&self.args.flag_gas_floor_target)),
			gas_ceil_target: try!(to_u256(&self.args.flag_gas_cap)),
			transactions_limit: self.args.flag_tx_queue_size,
		};

		Ok(extras)
	}

	fn author(&self) -> Result<Address, String> {
		to_address(self.args.flag_etherbase.clone().or(self.args.flag_author.clone()))
	}

	fn format(&self) -> Result<Option<DataFormat>, String> {
		match self.args.flag_format {
			Some(ref f) => Ok(Some(try!(f.parse()))),
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
			"morden".to_owned()
		} else {
			self.args.flag_chain.clone()
		}
	}

	fn max_peers(&self) -> u32 {
		let peers = self.args.flag_max_peers as u32;
		max(self.min_peers(), peers)
	}

	fn min_peers(&self) -> u32 {
		self.args.flag_peers.unwrap_or(self.args.flag_min_peers) as u32
	}

	fn work_notify(&self) -> Vec<String> {
		self.args.flag_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	fn accounts_config(&self) -> Result<AccountsConfig, String> {
		let cfg = AccountsConfig {
			iterations: self.args.flag_keys_iterations,
			testnet: self.args.flag_testnet,
			password_files: self.args.flag_password.clone(),
			unlocked_accounts: try!(to_addresses(&self.args.flag_unlock)),
		};

		Ok(cfg)
	}

	fn miner_options(&self) -> Result<MinerOptions, String> {
		let reseal = try!(self.args.flag_reseal_on_txs.parse::<ResealPolicy>());

		let options = MinerOptions {
			new_work_notify: self.work_notify(),
			force_sealing: self.args.flag_force_sealing,
			reseal_on_external_tx: reseal.external,
			reseal_on_own_tx: reseal.own,
			tx_gas_limit: match self.args.flag_tx_gas_limit {
				Some(ref d) => try!(to_u256(d)),
				None => U256::max_value(),
			},
			tx_queue_size: self.args.flag_tx_queue_size,
			tx_queue_gas_limit: try!(to_gas_limit(&self.args.flag_tx_queue_gas)),
			tx_queue_strategy: try!(to_queue_strategy(&self.args.flag_tx_queue_strategy)),
			pending_set: try!(to_pending_set(&self.args.flag_relay_set)),
			reseal_min_period: Duration::from_millis(self.args.flag_reseal_min_period),
			work_queue_size: self.args.flag_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
		};

		Ok(options)
	}

	fn signer_config(&self) -> SignerConfiguration {
		SignerConfiguration {
			enabled: self.signer_enabled(),
			port: self.args.flag_signer_port,
			interface: self.signer_interface(),
			signer_path: self.directories().signer,
			skip_origin_validation: self.args.flag_signer_no_validation,
		}
	}

	fn dapps_config(&self) -> DappsConfiguration {
		DappsConfiguration {
			enabled: self.dapps_enabled(),
			interface: self.dapps_interface(),
			port: self.args.flag_dapps_port,
			hosts: self.dapps_hosts(),
			user: self.args.flag_dapps_user.clone(),
			pass: self.args.flag_dapps_pass.clone(),
			dapps_path: self.directories().dapps,
		}
	}

	fn gas_pricer_config(&self) -> Result<GasPricerConfig, String> {
		if let Some(d) = self.args.flag_gasprice.as_ref() {
			return Ok(GasPricerConfig::Fixed(try!(to_u256(d))));
		}

		let usd_per_tx = try!(to_price(&self.args.flag_usd_per_tx));
		if "auto" == self.args.flag_usd_per_eth.as_str() {
			return Ok(GasPricerConfig::Calibrated {
				usd_per_tx: usd_per_tx,
				recalibration_period: try!(to_duration(self.args.flag_price_update_period.as_str())),
			});
		}

		let usd_per_eth = try!(to_price(&self.args.flag_usd_per_eth));
		let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
		let gas_per_tx: f32 = 21000.0;
		let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;

		info!(
			"Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)",
			Colour::White.bold().paint(format!("US${}", usd_per_eth)),
			Colour::Yellow.bold().paint(format!("{}", wei_per_gas))
		);

		Ok(GasPricerConfig::Fixed(U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()))
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
				let mut node_file = try!(File::open(path).map_err(|e| format!("Error opening reserved nodes file: {}", e)));
				try!(node_file.read_to_string(&mut buffer).map_err(|_| "Error reading reserved node file"));
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
			let host = try!(host.parse().map_err(|_| format!("Invalid host given with `--nat extip:{}`", host)));
			Some(SocketAddr::new(host, port))
		} else {
			None
		};
		Ok((listen_address, public_address))
	}

	fn net_config(&self) -> Result<NetworkConfiguration, String> {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = try!(to_bootnodes(&self.args.flag_bootnodes));
		let (listen, public) = try!(self.net_addresses());
		ret.listen_address = listen.map(|l| format!("{}", l));
		ret.public_address = public.map(|p| format!("{}", p));
		ret.use_secret = self.args.flag_node_key.as_ref().map(|s| s.parse::<Secret>().unwrap_or_else(|_| s.sha3()));
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.max_peers = self.max_peers();
		ret.min_peers = self.min_peers();
		let mut net_path = PathBuf::from(self.directories().db);
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret.reserved_nodes = try!(self.init_reserved_nodes());
		ret.allow_non_reserved = !self.args.flag_reserved_only;
		Ok(ret)
	}

	fn network_id(&self) -> Result<Option<U256>, String> {
		let net_id = self.args.flag_network_id.as_ref().or(self.args.flag_networkid.as_ref());
		match net_id {
			Some(id) => Ok(Some(try!(to_u256(id)))),
			None => Ok(None),
		}
	}

	fn rpc_apis(&self) -> String {
		self.args.flag_rpcapi.clone().unwrap_or(self.args.flag_jsonrpc_apis.clone())
	}

	fn rpc_cors(&self) -> Option<Vec<String>> {
		let cors = self.args.flag_jsonrpc_cors.clone().or(self.args.flag_rpccorsdomain.clone());
		cors.map(|c| c.split(',').map(|s| s.to_owned()).collect())
	}

	fn rpc_hosts(&self) -> Option<Vec<String>> {
		match self.args.flag_jsonrpc_hosts.as_ref() {
			"none" => return Some(Vec::new()),
			"all" => return None,
			_ => {}
		}
		let hosts = self.args.flag_jsonrpc_hosts.split(',').map(|h| h.into()).collect();
		Some(hosts)
	}

	fn dapps_hosts(&self) -> Option<Vec<String>> {
		match self.args.flag_dapps_hosts.as_ref() {
			"none" => return Some(Vec::new()),
			"all" => return None,
			_ => {}
		}
		let hosts = self.args.flag_dapps_hosts.split(',').map(|h| h.into()).collect();
		Some(hosts)
	}

	fn ipc_config(&self) -> Result<IpcConfiguration, String> {
		let conf = IpcConfiguration {
			enabled: !(self.args.flag_ipcdisable || self.args.flag_ipc_off || self.args.flag_no_ipc),
			socket_addr: self.ipc_path(),
			apis: try!(self.args.flag_ipcapi.clone().unwrap_or(self.args.flag_ipc_apis.clone()).parse()),
		};

		Ok(conf)
	}

	fn http_config(&self) -> Result<HttpConfiguration, String> {
		let conf = HttpConfiguration {
			enabled: !self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc,
			interface: self.rpc_interface(),
			port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
			apis: try!(self.rpc_apis().parse()),
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

	fn directories(&self) -> Directories {
		use util::path;

		let db_path = replace_home(self.args.flag_datadir.as_ref().unwrap_or(&self.args.flag_db_path));

		let keys_path = replace_home(
			if self.args.flag_testnet {
				"$HOME/.parity/testnet_keys"
			} else {
				&self.args.flag_keys_path
			}
		);

		let dapps_path = replace_home(&self.args.flag_dapps_path);
		let signer_path = replace_home(&self.args.flag_signer_path);

		if self.args.flag_geth  && !cfg!(windows) {
			let geth_root  = if self.args.flag_testnet { path::ethereum::test() } else {  path::ethereum::default() };
			::std::fs::create_dir_all(geth_root.as_path()).unwrap_or_else(
				|e| warn!("Failed to create '{}' for geth mode: {}", &geth_root.to_str().unwrap(), e));
		}

		if cfg!(feature = "ipc") && !cfg!(feature = "windows") {
			let mut path_buf = PathBuf::from(db_path.clone());
			path_buf.push("ipc");
			let ipc_path = path_buf.to_str().unwrap();
			::std::fs::create_dir_all(ipc_path).unwrap_or_else(
				|e| warn!("Failed to directory '{}' for ipc sockets: {}", ipc_path, e)
			);
		}

		Directories {
			keys: keys_path,
			db: db_path,
			dapps: dapps_path,
			signer: signer_path,
		}
	}

	fn ipc_path(&self) -> String {
		if self.args.flag_geth {
			geth_ipc_path(self.args.flag_testnet)
		} else {
			parity_ipc_path(&self.args.flag_ipcpath.clone().unwrap_or(self.args.flag_ipc_path.clone()))
		}
	}

	fn signer_port(&self) -> Option<u16> {
		if !self.signer_enabled() {
			None
		} else {
			Some(self.args.flag_signer_port)
		}
	}

	fn signer_interface(&self) -> String {
		match self.args.flag_signer_interface.as_str() {
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn rpc_interface(&self) -> String {
		match self.network_settings().rpc_interface.as_str() {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn dapps_interface(&self) -> String {
		match self.args.flag_dapps_interface.as_str() {
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	fn dapps_enabled(&self) -> bool {
		!self.args.flag_dapps_off && !self.args.flag_no_dapps && cfg!(feature = "dapps")
	}

	fn signer_enabled(&self) -> bool {
		if self.args.flag_force_signer {
			return true;
		}

		let signer_disabled = self.args.flag_unlock.is_some() ||
			self.args.flag_geth ||
			self.args.flag_no_signer;

		!signer_disabled
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use cli::Args;
	use ethcore_rpc::NetworkSettings;
	use ethcore::client::{VMType, BlockID};
	use ethcore::miner::{MinerOptions, PrioritizationStrategy};
	use helpers::{replace_home, default_network_config};
	use run::RunCmd;
	use signer::Configuration as SignerConfiguration;
	use blockchain::{BlockchainCmd, ImportBlockchain, ExportBlockchain, DataFormat};
	use presale::ImportWallet;
	use account::{AccountCmd, NewAccount, ImportAccounts};
	use devtools::{RandomTempPath};
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
		assert_eq!(conf.into_command().unwrap(), Cmd::Version);
	}

	#[test]
	fn test_command_account_new() {
		let args = vec!["parity", "account", "new"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Account(AccountCmd::New(NewAccount {
			iterations: 10240,
			path: replace_home("$HOME/.parity/keys"),
			password_file: None,
		})));
	}

	#[test]
	fn test_command_account_list() {
		let args = vec!["parity", "account", "list"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Account(
			AccountCmd::List(replace_home("$HOME/.parity/keys")))
		);
	}

	#[test]
	fn test_command_account_import() {
		let args = vec!["parity", "account", "import", "my_dir", "another_dir"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Account(AccountCmd::Import(ImportAccounts {
			from: vec!["my_dir".into(), "another_dir".into()],
			to: replace_home("$HOME/.parity/keys"),
		})));
	}

	#[test]
	fn test_command_wallet_import() {
		let args = vec!["parity", "wallet", "import", "my_wallet.json", "--password", "pwd"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::ImportPresaleWallet(ImportWallet {
			iterations: 10240,
			path: replace_home("$HOME/.parity/keys"),
			wallet_path: "my_wallet.json".into(),
			password_file: Some("pwd".into()),
		}));
	}

	#[test]
	fn test_command_blockchain_import() {
		let args = vec!["parity", "import", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Blockchain(BlockchainCmd::Import(ImportBlockchain {
			spec: Default::default(),
			logger_config: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			format: Default::default(),
			pruning: Default::default(),
			pruning_history: 64,
			compaction: Default::default(),
			wal: true,
			mode: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			vm_type: VMType::Interpreter,
		})));
	}

	#[test]
	fn test_command_blockchain_export() {
		let args = vec!["parity", "export", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			logger_config: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 64,
			format: Default::default(),
			compaction: Default::default(),
			wal: true,
			mode: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockID::Number(1),
			to_block: BlockID::Latest,
		})));
	}

	#[test]
	fn test_command_blockchain_export_with_custom_format() {
		let args = vec!["parity", "export", "--format", "hex", "blockchain.json"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: Default::default(),
			logger_config: Default::default(),
			cache_config: Default::default(),
			dirs: Default::default(),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			pruning_history: 64,
			format: Some(DataFormat::Hex),
			compaction: Default::default(),
			wal: true,
			mode: Default::default(),
			tracing: Default::default(),
			fat_db: Default::default(),
			from_block: BlockID::Number(1),
			to_block: BlockID::Latest,
		})));
	}

	#[test]
	fn test_command_signer_new_token() {
		let args = vec!["parity", "signer", "new-token"];
		let conf = parse(&args);
		let expected = replace_home("$HOME/.parity/signer");
		assert_eq!(conf.into_command().unwrap(), Cmd::SignerToken(expected));
	}

	#[test]
	fn test_run_cmd() {
		let args = vec!["parity"];
		let conf = parse(&args);
		assert_eq!(conf.into_command().unwrap(), Cmd::Run(RunCmd {
			cache_config: Default::default(),
			dirs: Default::default(),
			spec: Default::default(),
			pruning: Default::default(),
			pruning_history: 64,
			daemon: None,
			logger_config: Default::default(),
			miner_options: Default::default(),
			http_conf: Default::default(),
			ipc_conf: Default::default(),
			net_conf: default_network_config(),
			network_id: None,
			warp_sync: false,
			acc_conf: Default::default(),
			gas_pricer: Default::default(),
			miner_extras: Default::default(),
			mode: Default::default(),
			tracing: Default::default(),
			compaction: Default::default(),
			wal: true,
			vm_type: Default::default(),
			enable_network: true,
			geth_compatibility: false,
			signer_port: Some(8180),
			net_settings: Default::default(),
			dapps_conf: Default::default(),
			signer_conf: Default::default(),
			ui: false,
			name: "".into(),
			custom_bootnodes: false,
			fat_db: Default::default(),
			no_periodic_snapshot: false,
		}));
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
	fn should_parse_network_settings() {
		// given

		// when
		let conf = parse(&["parity", "--testnet", "--identity", "testname"]);

		// then
		assert_eq!(conf.network_settings(), NetworkSettings {
			name: "testname".to_owned(),
			chain: "morden".to_owned(),
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
	fn should_disable_signer_in_geth_compat() {
		// given

		// when
		let conf0 = parse(&["parity", "--geth"]);
		let conf1 = parse(&["parity", "--geth", "--force-signer"]);

		// then
		assert_eq!(conf0.signer_enabled(), false);
		assert_eq!(conf1.signer_enabled(), true);
	}

	#[test]
	fn should_disable_signer_when_account_is_unlocked() {
		// given

		// when
		let conf0 = parse(&["parity", "--unlock", "0x0"]);

		// then
		assert_eq!(conf0.signer_enabled(), false);
	}

	#[test]
	fn should_parse_signer_configration() {
		// given

		// when
		let conf0 = parse(&["parity", "--signer-path", "signer"]);
		let conf1 = parse(&["parity", "--signer-path", "signer", "--signer-no-validation"]);
		let conf2 = parse(&["parity", "--signer-path", "signer", "--signer-port", "3123"]);
		let conf3 = parse(&["parity", "--signer-path", "signer", "--signer-interface", "test"]);

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
	fn should_not_bail_on_empty_line_in_reserved_peers() {
		let temp = RandomTempPath::new();
		create_dir(temp.as_str().to_owned()).unwrap();
		let filename = temp.as_str().to_owned() + "/peers";
		File::create(filename.clone()).unwrap().write_all(b"  \n\t\n").unwrap();
		let args = vec!["parity", "--reserved-peers", &filename];
		let conf = Configuration::parse(&args).unwrap();
		assert!(conf.init_reserved_nodes().is_ok());
	}
}

