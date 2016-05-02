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

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::{SocketAddr, IpAddr};
use std::path::PathBuf;
use cli::{USAGE, Args};
use docopt::Docopt;

use die::*;
use util::*;
use util::keys::store::AccountService;
use util::network_settings::NetworkSettings;
use ethcore::client::{append_path, get_db_path, ClientConfig, Switch};
use ethcore::ethereum;
use ethcore::spec::Spec;
use ethsync::SyncConfig;
use price_info::PriceInfo;

pub struct Configuration {
	pub args: Args
}

impl Configuration {
	pub fn parse() -> Self {
		Configuration {
			args: Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit()),
		}
	}

	fn net_port(&self) -> u16 {
		self.args.flag_port
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

	pub fn path(&self) -> String {
		let d = self.args.flag_datadir.as_ref().unwrap_or(&self.args.flag_db_path);
		d.replace("$HOME", env::home_dir().unwrap().to_str().unwrap())
	}

	pub fn author(&self) -> Address {
		let d = self.args.flag_etherbase.as_ref().unwrap_or(&self.args.flag_author);
		Address::from_str(clean_0x(d)).unwrap_or_else(|_| {
			die!("{}: Invalid address for --author. Must be 40 hex characters, with or without the 0x at the beginning.", d)
		})
	}

	pub fn gas_floor_target(&self) -> U256 {
		let d = &self.args.flag_gas_floor_target;
		U256::from_dec_str(d).unwrap_or_else(|_| {
			die!("{}: Invalid target gas floor given. Must be a decimal unsigned 256-bit number.", d)
		})
	}

	pub fn gas_price(&self) -> U256 {
		match self.args.flag_gasprice.as_ref() {
			Some(d) => {
				U256::from_dec_str(d).unwrap_or_else(|_| {
					die!("{}: Invalid gas price given. Must be a decimal unsigned 256-bit number.", d)
				})
			}
			_ => {
				let usd_per_tx: f32 = FromStr::from_str(&self.args.flag_usd_per_tx).unwrap_or_else(|_| {
					die!("{}: Invalid basic transaction price given in USD. Must be a decimal number.", self.args.flag_usd_per_tx)
				});
				let usd_per_eth = match self.args.flag_usd_per_eth.as_str() {
					"etherscan" => PriceInfo::get().map_or_else(|| {
						die!("Unable to retrieve USD value of ETH from etherscan. Rerun with a different value for --usd-per-eth.")
					}, |x| x.ethusd),
					x => FromStr::from_str(x).unwrap_or_else(|_| die!("{}: Invalid ether price given in USD. Must be a decimal number.", x))
				};
				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!("Using a conversion rate of Ξ1 = US${} ({} wei/gas)", usd_per_eth, wei_per_gas);
				U256::from_dec_str(&format!("{:.0}", wei_per_gas)).unwrap()
			}
		}
	}

	pub fn extra_data(&self) -> Bytes {
		match self.args.flag_extradata.as_ref().or(self.args.flag_extra_data.as_ref()) {
			Some(ref x) if x.len() <= 32 => x.as_bytes().to_owned(),
			None => version_data(),
			Some(ref x) => { die!("{}: Extra data must be at most 32 characters.", x); }
		}
	}

	pub fn keys_path(&self) -> String {
		self.args.flag_keys_path.replace("$HOME", env::home_dir().unwrap().to_str().unwrap())
	}

	pub fn spec(&self) -> Spec {
		match self.chain().as_str() {
			"frontier" | "homestead" | "mainnet" => ethereum::new_frontier(),
			"morden" | "testnet" => ethereum::new_morden(),
			"olympic" => ethereum::new_olympic(),
			f => Spec::load(contents(f).unwrap_or_else(|_| {
				die!("{}: Couldn't read chain specification file. Sure it exists?", f)
			}).as_ref()),
		}
	}

	pub fn normalize_enode(e: &str) -> Option<String> {
		if is_valid_node_url(e) {
			Some(e.to_owned())
		} else {
			None
		}
	}

	pub fn init_nodes(&self, spec: &Spec) -> Vec<String> {
		match self.args.flag_bootnodes {
			Some(ref x) if !x.is_empty() => x.split(',').map(|s| {
				Self::normalize_enode(s).unwrap_or_else(|| {
					die!("{}: Invalid node address format given for a boot node.", s)
				})
			}).collect(),
			Some(_) => Vec::new(),
			None => spec.nodes().clone(),
		}
	}

	pub fn net_addresses(&self) -> (Option<SocketAddr>, Option<SocketAddr>) {
		let port = self.net_port();
		let listen_address = Some(SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port));
		let public_address = if self.args.flag_nat.starts_with("extip:") {
			let host = &self.args.flag_nat[6..];
			let host = IpAddr::from_str(host).unwrap_or_else(|_| die!("Invalid host given with `--nat extip:{}`", host));
			Some(SocketAddr::new(host, port))
		} else {
			listen_address
		};
		(listen_address, public_address)
	}

	pub fn net_settings(&self, spec: &Spec) -> NetworkConfiguration {
		let mut ret = NetworkConfiguration::new();
		ret.nat_enabled = self.args.flag_nat == "any" || self.args.flag_nat == "upnp";
		ret.boot_nodes = self.init_nodes(spec);
		let (listen, public) = self.net_addresses();
		ret.listen_address = listen;
		ret.public_address = public;
		ret.use_secret = self.args.flag_node_key.as_ref().map(|s| Secret::from_str(&s).unwrap_or_else(|_| s.sha3()));
		ret.discovery_enabled = !self.args.flag_no_discovery && !self.args.flag_nodiscover;
		ret.ideal_peers = self.max_peers();
		let mut net_path = PathBuf::from(&self.path());
		net_path.push("network");
		ret.config_path = Some(net_path.to_str().unwrap().to_owned());
		ret
	}

	pub fn find_best_db(&self, spec: &Spec) -> Option<journaldb::Algorithm> {
		let mut ret = None;
		let mut latest_era = None;
		let jdb_types = [journaldb::Algorithm::Archive, journaldb::Algorithm::EarlyMerge, journaldb::Algorithm::OverlayRecent, journaldb::Algorithm::RefCounted];
		for i in jdb_types.into_iter() {
			let db = journaldb::new(&append_path(&get_db_path(&Path::new(&self.path()), *i, spec.genesis_header().hash()), "state"), *i);
			trace!(target: "parity", "Looking for best DB: {} at {:?}", i, db.latest_era());
			match (latest_era, db.latest_era()) {
				(Some(best), Some(this)) if best >= this => {}
				(_, None) => {}
				(_, Some(this)) => {
					latest_era = Some(this);
					ret = Some(*i);
				}
			}
		}
		ret
	}

	pub fn client_config(&self, spec: &Spec) -> ClientConfig {
		let mut client_config = ClientConfig::default();
		match self.args.flag_cache {
			Some(mb) => {
				client_config.blockchain.max_cache_size = mb * 1024 * 1024;
				client_config.blockchain.pref_cache_size = client_config.blockchain.max_cache_size * 3 / 4;
			}
			None => {
				client_config.blockchain.pref_cache_size = self.args.flag_cache_pref_size;
				client_config.blockchain.max_cache_size = self.args.flag_cache_max_size;
			}
		}
		client_config.tracing.enabled = match self.args.flag_tracing.as_str() {
			"auto" => Switch::Auto,
			"on" => Switch::On,
			"off" => Switch::Off,
			_ => { die!("Invalid tracing method given!") }
		};
		client_config.pruning = match self.args.flag_pruning.as_str() {
			"archive" => journaldb::Algorithm::Archive,
			"light" => journaldb::Algorithm::EarlyMerge,
			"fast" => journaldb::Algorithm::OverlayRecent,
			"basic" => journaldb::Algorithm::RefCounted,
			"auto" => self.find_best_db(spec).unwrap_or(journaldb::Algorithm::OverlayRecent),
			_ => { die!("Invalid pruning method given."); }
		};
		trace!(target: "parity", "Using pruning strategy of {}", client_config.pruning);
		client_config.name = self.args.flag_identity.clone();
		client_config.queue.max_mem_use = self.args.flag_queue_max_size;
		client_config
	}

	pub fn sync_config(&self, spec: &Spec) -> SyncConfig {
		let mut sync_config = SyncConfig::default();
		sync_config.network_id = self.args.flag_network_id.as_ref().or(self.args.flag_networkid.as_ref()).map_or(spec.network_id(), |id| {
			U256::from_str(id).unwrap_or_else(|_| die!("{}: Invalid index given with --network-id/--networkid", id))
		});
		sync_config
	}

	pub fn account_service(&self) -> AccountService {
		// Secret Store
		let passwords = self.args.flag_password.iter().flat_map(|filename| {
			BufReader::new(&File::open(filename).unwrap_or_else(|_| die!("{} Unable to read password file. Ensure it exists and permissions are correct.", filename)))
				.lines()
				.map(|l| l.unwrap())
				.collect::<Vec<_>>()
				.into_iter()
		}).collect::<Vec<_>>();
		let account_service = AccountService::new_in(Path::new(&self.keys_path()));
		if let Some(ref unlocks) = self.args.flag_unlock {
			for d in unlocks.split(',') {
				let a = Address::from_str(clean_0x(&d)).unwrap_or_else(|_| {
					die!("{}: Invalid address for --unlock. Must be 40 hex characters, without the 0x at the beginning.", d)
				});
				if passwords.iter().find(|p| account_service.unlock_account_no_expire(&a, p).is_ok()).is_none() {
					die!("No password given to unlock account {}. Pass the password using `--password`.", a);
				}
			}
		}
		account_service
	}

	pub fn rpc_apis(&self) -> String {
		self.args.flag_rpcapi.clone().unwrap_or(self.args.flag_jsonrpc_apis.clone())
	}

	pub fn rpc_cors(&self) -> Option<String> {
		self.args.flag_jsonrpc_cors.clone().or(self.args.flag_rpccorsdomain.clone())
	}

	pub fn network_settings(&self) -> NetworkSettings {
		NetworkSettings {
			name: self.args.flag_identity.clone(),
			chain: self.chain(),
			max_peers: self.max_peers(),
			network_port: self.net_port(),
			rpc_enabled: self.args.flag_rpc || self.args.flag_jsonrpc,
			rpc_interface: self.args.flag_rpcaddr.clone().unwrap_or(self.args.flag_jsonrpc_interface.clone()),
			rpc_port: self.args.flag_rpcport.unwrap_or(self.args.flag_jsonrpc_port),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use cli::USAGE;
	use docopt::Docopt;
	use util::network_settings::NetworkSettings;

	fn parse(args: &[&str]) -> Configuration {
		Configuration {
			args: Docopt::new(USAGE).unwrap().argv(args).decode().unwrap(),
		}
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
			rpc_enabled: false,
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
			assert_eq!(conf.rpc_cors(), Some("*".to_owned()));
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

