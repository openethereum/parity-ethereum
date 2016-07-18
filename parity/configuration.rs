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

use std::{env, fs};
use std::fs::File;
use std::time::Duration;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use cli::{USAGE, Args};
use docopt::{Docopt, Error as DocoptError};
use util::{Hashable, journaldb, NetworkConfiguration, kvdb, U256, Uint, is_valid_node_url, Bytes, version_data, Secret, path};
use util::network_settings::NetworkSettings;
use util::log::Colour;
use ethcore::account_provider::AccountProvider;
use ethcore::client::{append_path, ClientConfig, VMType};
use ethcore::miner::{MinerOptions, GasPricer, GasPriceCalibratorOptions};
use ethcore::spec::Spec;
use ethsync::SyncConfig;
use rpc::{IpcConfiguration, HttpConfiguration};
use commands::{Cmd, AccountCmd, ImportWallet, NewAccount, ImportAccounts, BlockchainCmd, ImportBlockchain, ExportBlockchain};
use cache::CacheConfig;
use helpers::{to_duration, to_mode, to_block_id, to_u256, to_pending_set, to_price, flush_stdout, replace_home,
geth_ipc_path, parity_ipc_path};
use params::{SpecType, ResealPolicy};
use setup_log::LoggerConfig;
use dir::Directories;
use RunCmd;

/// Should be used to read password.
pub trait PasswordReader {
	/// Prompts user for password.
	fn prompt(&self) -> Result<String, String>;

	/// Loads user password from file.
	fn file(&self, file: &str) -> Result<String, String>;
}

/// Reads password from standard IO.
#[derive(Debug, PartialEq)]
pub struct IOPasswordReader;

impl PasswordReader for IOPasswordReader {
	fn prompt(&self) -> Result<String, String> {
		use rpassword::read_password;

		println!("Please note that password is NOT RECOVERABLE.");
		print!("Type password: ");
		flush_stdout();

		let password = read_password().unwrap();

		print!("Repeat password: ");
		flush_stdout();

		let password_repeat = read_password().unwrap();

		if password != password_repeat {
			return Err("Passwords do not match!".into());
		}

		Ok(password)
	}

	fn file(&self, file: &str) -> Result<String, String> {
		unimplemented!();
	}
}

#[derive(Debug, PartialEq)]
pub struct Configuration {
	pub args: Args,
}

impl Configuration {
	pub fn parse<S, I>(command: I) -> Result<Self, DocoptError> where I: IntoIterator<Item=S>, S: AsRef<str> {
		let args = try!(Docopt::new(USAGE).and_then(|d| d.argv(command).decode()));

		let config = Configuration {
			args: args,
		};

		Ok(config)
	}

	pub fn into_command(self, password: &PasswordReader) -> Result<Cmd, String> {
		let dirs = self.directories();
		let logger_config = LoggerConfig {
			mode: None,
			color: false,
			file: None,
		};
		let pruning = try!(self.args.flag_pruning.parse());
		let vm_type = try!(self.vm_type());
		let mode = try!(to_mode(&self.args.flag_mode, self.args.flag_mode_timeout, self.args.flag_mode_alarm));
		let miner_options = try!(self.miner_options());

		let http_conf = try!(self.http_config());
		let ipc_conf = try!(self.ipc_config());

		let cmd = if self.args.flag_version {
			Cmd::Version
		} else if self.args.cmd_signer {
			Cmd::SignerToken(dirs.signer)
		} else if self.args.cmd_account {
			let account_cmd = if self.args.cmd_new {
				let new_acc = NewAccount {
					iterations: self.args.flag_keys_iterations,
					path: dirs.keys,
					password: try!(password.prompt()),
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
		} else if self.args.cmd_wallet {
			let presale_cmd = ImportWallet {
				iterations: self.args.flag_keys_iterations,
				path: dirs.keys,
				wallet_path: self.args.arg_path.first().unwrap().clone(),
				password: try!(password.file(self.args.flag_password.first().unwrap())),
			};
			Cmd::ImportPresaleWallet(presale_cmd)
		} else if self.args.cmd_import {
			let import_cmd = ImportBlockchain {
				spec: try!(self.chain().parse()),
				logger_config: logger_config,
				cache_config: self.cache_config(),
				db_path: dirs.db.clone(),
				file_path: self.args.arg_file.clone(),
				format: None,
				pruning: pruning,
				compaction: try!(self.args.flag_db_compaction.parse()),
				mode: mode,
				tracing: try!(self.args.flag_tracing.parse()),
				vm_type: vm_type,
			};
			Cmd::Blockchain(BlockchainCmd::Import(import_cmd))
		} else if self.args.cmd_export {
			let export_cmd = ExportBlockchain {
				spec: try!(self.chain().parse()),
				logger_config: logger_config,
				cache_config: self.cache_config(),
				db_path: dirs.db.clone(),
				file_path: self.args.arg_file.clone(),
				format: None,
				pruning: pruning,
				compaction: try!(self.args.flag_db_compaction.parse()),
				mode: mode,
				tracing: try!(self.args.flag_tracing.parse()),
				from_block: try!(to_block_id(&self.args.flag_from)),
				to_block: try!(to_block_id(&self.args.flag_to)),
			};
			Cmd::Blockchain(BlockchainCmd::Export(export_cmd))
		} else {
			let daemon = if self.args.cmd_daemon {
				Some(self.args.arg_pid_file.clone())
			} else {
				None
			};

			let run_cmd = RunCmd {
				directories: dirs,
				spec: try!(self.chain().parse()),
				pruning: pruning,
				daemon: daemon,
				logger_config: logger_config,
				miner_options: miner_options,
				http_conf: http_conf,
				ipc_conf: ipc_conf,
			};
			Cmd::Run(run_cmd)
		};

		Ok(cmd)
	}


	fn vm_type(&self) -> Result<VMType, String> {
		if self.args.flag_jitvm {
			VMType::jit().ok_or("Parity is built without the JIT EVM.".into())
		} else {
			Ok(VMType::Interpreter)
		}
	}

	fn cache_config(&self) -> CacheConfig {
		match self.args.flag_cache_size {
			Some(size) => CacheConfig::new_with_total_cache_size(size),
			None => CacheConfig {
				rocksdb: self.args.flag_cache_size_db,
				blockchain: self.args.flag_cache_size_blocks,
				queue: self.args.flag_cache_size_queue,
			}
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
		self.args.flag_maxpeers.unwrap_or(self.args.flag_peers) as u32
	}

	fn work_notify(&self) -> Vec<String> {
		self.args.flag_notify_work.as_ref().map_or_else(Vec::new, |s| s.split(',').map(|s| s.to_owned()).collect())
	}

	pub fn miner_options(&self) -> Result<MinerOptions, String> {
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
			pending_set: try!(to_pending_set(&self.args.flag_relay_set)),
			reseal_min_period: Duration::from_millis(self.args.flag_reseal_min_period),
			work_queue_size: self.args.flag_work_queue_size,
			enable_resubmission: !self.args.flag_remove_solved,
		};

		Ok(options)
	}

	pub fn gas_pricer(&self) -> Result<GasPricer, String> {
		match self.args.flag_gasprice.as_ref() {
			Some(d) => Ok(GasPricer::Fixed(try!(to_u256(d)))),
			_ => {
				let usd_per_tx = try!(to_price(&self.args.flag_usd_per_tx));
				match self.args.flag_usd_per_eth.as_str() {
					"auto" => {
						Ok(GasPricer::new_calibrated(GasPriceCalibratorOptions {
							usd_per_tx: usd_per_tx,
							recalibration_period: try!(to_duration(self.args.flag_price_update_period.as_str())),
						}))
					},
					x => {
						let usd_per_eth = try!(to_price(x));
						let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
						let gas_per_tx: f32 = 21000.0;
						let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
						info!("Using a fixed conversion rate of Ξ1 = {} ({} wei/gas)", Colour::White.bold().paint(format!("US${}", usd_per_eth)), Colour::Yellow.bold().paint(format!("{}", wei_per_gas)));
						Ok(GasPricer::Fixed(U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()))
					}
				}
			}
		}
	}

	pub fn extra_data(&self) -> Result<Bytes, String> {
		match self.args.flag_extradata.as_ref().or(self.args.flag_extra_data.as_ref()) {
			Some(ref x) if x.len() <= 32 => Ok(x.as_bytes().to_owned()),
			None => Ok(version_data()),
			Some(ref x) => Err("Extra data must be at most 32 characters".into()),
		}
	}

	pub fn spec(&self) -> Spec {
		unimplemented!();
		//match self.chain().as_str() {
			//"frontier" | "homestead" | "mainnet" => ethereum::new_frontier(),
			//"morden" | "testnet" => ethereum::new_morden(),
			//"olympic" => ethereum::new_olympic(),
			//f => Spec::load(contents(f).unwrap_or_else(|_| {
				//die!("{}: Couldn't read chain specification file. Sure it exists?", f)
			//}).as_ref()),
		//}
	}

	pub fn init_nodes(&self, spec: &Spec) -> Result<Vec<String>, String> {
		match self.args.flag_bootnodes {
			Some(ref x) if !x.is_empty() => x.split(',').map(|s| {
				if is_valid_node_url(s) {
					Ok(s.to_owned())
				} else {
					Err(format!("Invalid node address format given for a boot node: {}", s))
				}
			}).collect(),
			Some(_) => Ok(Vec::new()),
			None => Ok(spec.nodes().to_owned()),
		}
	}

	pub fn init_reserved_nodes(&self) -> Result<Vec<String>, String> {
		use std::fs::File;

		match self.args.flag_reserved_peers {
			Some(ref path) => {
				let mut buffer = String::new();
				let mut node_file = try!(File::open(path).map_err(|e| format!("Error opening reserved nodes file: {}", e)));
				try!(node_file.read_to_string(&mut buffer).map_err(|_| "Error reading reserved node file"));
				if let Some(invalid) = buffer.lines().find(|s| !is_valid_node_url(s)) {
					Err(format!("Invalid node address format given for a boot node: {}", invalid))
				} else {
					Ok(buffer.lines().map(|s| s.to_owned()).collect())
				}
			},
			None => Ok(Vec::new())
		}
	}

	pub fn net_addresses(&self) -> Result<(Option<SocketAddr>, Option<SocketAddr>), String> {
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

	pub fn net_settings(&self, spec: &Spec) -> Result<NetworkConfiguration, String> {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = try!(self.init_nodes(spec));
		let (listen, public) = try!(self.net_addresses());
		ret.listen_address = listen;
		ret.public_address = public;
		ret.use_secret = self.args.flag_node_key.as_ref().map(|s| s.parse::<Secret>().unwrap_or_else(|_| s.sha3()));
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.ideal_peers = self.max_peers();
		let mut net_path = PathBuf::from(self.directories().db);
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret.reserved_nodes = try!(self.init_reserved_nodes());

		if self.args.flag_reserved_only {
			ret.non_reserved_mode = ::util::network::NonReservedPeerMode::Deny;
		}

		Ok(ret)
	}

	pub fn client_config(&self, spec: &Spec) -> ClientConfig {
		/*
		trace!(target: "parity", "Using pruning strategy of {}", client_config.pruning);
		client_config.name = self.args.flag_identity.clone();
		client_config.queue.max_mem_use = self.args.flag_queue_max_size;
		client_config
		*/
		unimplemented!();
	}

	pub fn sync_config(&self, spec: &Spec) -> Result<SyncConfig, String> {
		let mut sync_config = SyncConfig::default();
		let net_id = self.args.flag_network_id.as_ref().or(self.args.flag_networkid.as_ref());
		sync_config.network_id = match net_id {
			Some(id) => try!(to_u256(id)),
			None => spec.network_id()
		};

		Ok(sync_config)
	}

	pub fn account_service(&self) -> AccountProvider {
		/*
		use ethcore::ethstore::{import_accounts, EthStore};
		use ethcore::ethstore::dir::{GethDirectory, DirectoryType, DiskDirectory};

		// Secret Store
		let passwords = self.args.flag_password.iter().flat_map(|filename| {
			BufReader::new(&File::open(filename).unwrap_or_else(|_| die!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename)))
				.lines()
				.map(|l| l.unwrap())
				.collect::<Vec<_>>()
				.into_iter()
		}).collect::<Vec<_>>();

		if !self.args.flag_no_import_keys {
			let dir_type = if self.args.flag_testnet {
				DirectoryType::Testnet
			} else {
				DirectoryType::Main
			};

			let from = GethDirectory::open(dir_type);
			let to = DiskDirectory::create(self.keys_path()).unwrap();
			// ignore error, cause geth may not exist
			let _ = import_accounts(&from, &to);
		}

		let dir = Box::new(DiskDirectory::create(self.keys_path()).unwrap());
		let iterations = self.keys_iterations();
		let account_service = AccountProvider::new(Box::new(EthStore::open_with_iterations(dir, iterations).unwrap()));

		if let Some(ref unlocks) = self.args.flag_unlock {
			for d in unlocks.split(',') {
				let a = Address::from_str(clean_0x(d)).unwrap_or_else(|_| {
					die!("{}: Invalid address for --unlock. Must be 40 hex characters, without the 0x at the beginning.", d)
				});
				if passwords.iter().find(|p| account_service.unlock_account_permanently(a, (*p).clone()).is_ok()).is_none() {
					die!("No password given to unlock account {}. Pass the password using `--password`.", a);
				}
			}
		}
		account_service
		*/
		unimplemented!();
	}

	pub fn rpc_apis(&self) -> String {
		self.args.flag_rpcapi.clone().unwrap_or(self.args.flag_jsonrpc_apis.clone())
	}

	pub fn rpc_cors(&self) -> Vec<String> {
		let cors = self.args.flag_jsonrpc_cors.clone().or(self.args.flag_rpccorsdomain.clone());
		cors.map_or_else(Vec::new, |c| c.split(',').map(|s| s.to_owned()).collect())
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
			cors: self.rpc_cors(),
		};

		Ok(conf)
	}

	pub fn network_settings(&self) -> NetworkSettings {
		NetworkSettings {
			name: self.args.flag_identity.clone(),
			chain: self.chain(),
			max_peers: self.max_peers(),
			network_port: self.args.flag_port,
			rpc_enabled: !self.args.flag_jsonrpc_off && !self.args.flag_no_jsonrpc,
			rpc_interface: self.args.flag_rpcaddr.clone().unwrap_or(self.args.flag_jsonrpc_interface.clone()),
			rpc_port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
		}
	}

	pub fn directories(&self) -> Directories {
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

	pub fn have_color(&self) -> bool {
		!self.args.flag_no_color && !cfg!(windows)
	}

	pub fn signer_port(&self) -> Option<u16> {
		if !self.signer_enabled() {
			None
		} else {
			Some(self.args.flag_signer_port)
		}
	}

	pub fn rpc_interface(&self) -> String {
		match self.network_settings().rpc_interface.as_str() {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	pub fn dapps_interface(&self) -> String {
		match self.args.flag_dapps_interface.as_str() {
			"all" => "0.0.0.0",
			"local" => "127.0.0.1",
			x => x,
		}.into()
	}

	pub fn dapps_enabled(&self) -> bool {
		!self.args.flag_dapps_off && !self.args.flag_no_dapps && cfg!(feature = "dapps")
	}

	pub fn signer_enabled(&self) -> bool {
		(self.args.flag_unlock.is_none() && !self.args.flag_no_signer) ||
		self.args.flag_force_signer
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use cli::USAGE;
	use docopt::Docopt;
	use util::network_settings::NetworkSettings;
	use ethcore::client::{DatabaseCompactionProfile, Mode, Switch, VMType, BlockID};
	use commands::{Cmd, AccountCmd, NewAccount, ImportAccounts, ImportWallet, BlockchainCmd, ImportBlockchain, ExportBlockchain};
	use cache::CacheConfig;
	use params::{SpecType, Pruning};
	use helpers::replace_home;
	use setup_log::LoggerConfig;
	use dir::Directories;
	use RunCmd;

	#[derive(Debug, PartialEq)]
	struct TestPasswordReader(&'static str);

	impl PasswordReader for TestPasswordReader {
		fn prompt(&self) -> Result<String, String> {
			Ok(self.0.to_owned())
		}

		fn file(&self, _file: &str) -> Result<String, String> {
			Ok(self.0.to_owned())
		}
	}

	fn parse(args: &[&str]) -> Configuration {
		Configuration {
			args: Docopt::new(USAGE).unwrap().argv(args).decode().unwrap(),
		}
	}


	#[test]
	fn test_command_version() {
		let args = vec!["parity", "--version"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Version);
	}

	#[test]
	fn test_command_account_new() {
		let args = vec!["parity", "account", "new"];
		let mut conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Account(AccountCmd::New(NewAccount {
			iterations: 10240,
			path: replace_home("$HOME/.parity/keys"),
			password: "test".into(),
		})));
	}

	#[test]
	fn test_command_account_list() {
		let args = vec!["parity", "account", "list"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Account(
			AccountCmd::List(replace_home("$HOME/.parity/keys")))
		);
	}

	#[test]
	fn test_command_account_import() {
		let args = vec!["parity", "account", "import", "my_dir", "another_dir"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Account(AccountCmd::Import(ImportAccounts {
			from: vec!["my_dir".into(), "another_dir".into()],
			to: replace_home("$HOME/.parity/keys"),
		})));
	}

	#[test]
	fn test_command_wallet_import() {
		let args = vec!["parity", "wallet", "import", "my_wallet.json", "--password", "pwd"];
		let mut conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("content_of_pwd");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::ImportPresaleWallet(ImportWallet {
			iterations: 10240,
			path: replace_home("$HOME/.parity/keys"),
			wallet_path: "my_wallet.json".into(),
			password: "content_of_pwd".into(),
		}));
	}

	#[test]
	fn test_command_blockchain_import() {
		let args = vec!["parity", "import", "blockchain.json"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Blockchain(BlockchainCmd::Import(ImportBlockchain {
			spec: SpecType::Mainnet,
			logger_config: LoggerConfig {
				mode: None,
				color: false,
				file: None,
			},
			cache_config: Default::default(),
			db_path: replace_home("$HOME/.parity"),
			file_path: Some("blockchain.json".into()),
			format: None,
			pruning: Default::default(),
			compaction: Default::default(),
			mode: Default::default(),
			tracing: Default::default(),
			vm_type: VMType::Interpreter,
		})));
	}

	#[test]
	fn test_command_blockchain_export() {
		let args = vec!["parity", "export", "blockchain.json"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Blockchain(BlockchainCmd::Export(ExportBlockchain {
			spec: SpecType::Mainnet,
			logger_config: LoggerConfig {
				mode: None,
				color: false,
				file: None,
			},
			cache_config: Default::default(),
			db_path: replace_home("$HOME/.parity"),
			file_path: Some("blockchain.json".into()),
			pruning: Default::default(),
			format: Default::default(),
			compaction: Default::default(),
			mode: Default::default(),
			tracing: Default::default(),
			from_block: BlockID::Number(1),
			to_block: BlockID::Latest,
		})));
	}

	#[test]
	fn test_command_signer_new_token() {
		let args = vec!["parity", "signer", "new-token"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		let expected = replace_home("$HOME/.parity/signer");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::SignerToken(expected));
	}

	#[test]
	fn test_run_cmd() {
		let args = vec!["parity"];
		let conf = Configuration::parse(args).unwrap();
		let password = TestPasswordReader("test");
		assert_eq!(conf.into_command(&password).unwrap(), Cmd::Run(RunCmd {
			directories: Default::default(),
			spec: Default::default(),
			pruning: Default::default(),
			daemon: None,
			logger_config: LoggerConfig {
				mode: None,
				color: false,
				file: None,
			},
			miner_options: Default::default(),
			http_conf: Default::default(),
			ipc_conf: Default::default(),
		}));
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
			max_peers: 25,
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
			assert_eq!(conf.rpc_cors(), vec!["*".to_owned()]);
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
}

