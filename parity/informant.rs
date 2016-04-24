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

use std::sync::RwLock;
use std::ops::{Deref, DerefMut};
use ethsync::{EthSync, SyncProvider};
use ethcore::client::*;
use number_prefix::{binary_prefix, Standalone, Prefixed};

pub struct Informant {
	chain_info: RwLock<Option<BlockChainInfo>>,
	cache_info: RwLock<Option<BlockChainCacheSize>>,
	report: RwLock<Option<ClientReport>>,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
		}
	}
}

impl Informant {
	fn format_bytes(b: usize) -> String {
		match binary_prefix(b as f64) {
			Standalone(bytes)   => format!("{} bytes", bytes),
			Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
		}
	}

	pub fn tick(&self, client: &Client, sync: &EthSync) {
		// 5 seconds betwen calls. TODO: calculate this properly.
		let dur = 5usize;

		let chain_info = client.chain_info();
		let queue_info = client.queue_info();
		let cache_info = client.blockchain_cache_info();
		let sync_info = sync.status();

		let mut write_report = self.report.write().unwrap();
		let report = client.report();

		if let (_, _, &Some(ref last_report)) = (
			self.chain_info.read().unwrap().deref(),
			self.cache_info.read().unwrap().deref(),
			write_report.deref()
		) {
			println!("[ #{} {} ]---[ {} blk/s | {} tx/s | {} gas/s  //··· {}/{} peers, #{}, {}+{} queued ···// mem: {} db, {} chain, {} queue, {} sync ]",
				chain_info.best_block_number,
				chain_info.best_block_hash,
				(report.blocks_imported - last_report.blocks_imported) / dur,
				(report.transactions_applied - last_report.transactions_applied) / dur,
				(report.gas_processed - last_report.gas_processed) / From::from(dur),

				sync_info.num_active_peers,
				sync_info.num_peers,
				sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number),
				queue_info.unverified_queue_size,
				queue_info.verified_queue_size,

				Informant::format_bytes(report.state_db_mem),
				Informant::format_bytes(cache_info.total()),
				Informant::format_bytes(queue_info.mem_used),
				Informant::format_bytes(sync_info.mem_used),
			);
		}

		*self.chain_info.write().unwrap().deref_mut() = Some(chain_info);
		*self.cache_info.write().unwrap().deref_mut() = Some(cache_info);
		*write_report.deref_mut() = Some(report);
	}
}

