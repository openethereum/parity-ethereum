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
use util::Uint;
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

const TERM_RESET: &'static str = "\x1B[0m";
const _TERM_BLACK: &'static str = "\x1B[0;30m";
const _TERM_RED: &'static str = "\x1B[0;31m";
const _TERM_GREEN: &'static str = "\x1B[0;32m";
const _TERM_YELLOW: &'static str = "\x1B[0;33m";
const _TERM_BLUE: &'static str = "\x1B[0;34m";
const _TERM_MAGENTA: &'static str = "\x1B[0;35m";
const _TERM_CYAN: &'static str = "\x1B[0;36m";
const TERM_WHITE: &'static str = "\x1B[0;37m";
const _TERM_L_BLACK: &'static str = "\x1B[1;30m";
const _TERM_L_RED: &'static str = "\x1B[1;31m";
const TERM_L_GREEN: &'static str = "\x1B[1;32m";
const TERM_L_YELLOW: &'static str = "\x1B[1;33m";
const TERM_L_BLUE: &'static str = "\x1B[1;34m";
const TERM_L_MAGENTA: &'static str = "\x1B[1;35m";
const TERM_L_CYAN: &'static str = "\x1B[1;36m";
const TERM_L_WHITE: &'static str = "\x1B[1;37m";

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
			const FRAME: &'static str = " ";
			const LABEL_SPEED: &'static str = TERM_WHITE;
			const LABEL_SLASH: &'static str = TERM_WHITE;
			const LABEL_PEERS: &'static str = TERM_WHITE;
			const LABEL_PLUS: &'static str = TERM_WHITE;
			const LABEL_QUEUED: &'static str = TERM_WHITE;
			const LABEL_MEM: &'static str = TERM_WHITE;
			const CURRENT_NUMBER: &'static str = TERM_L_WHITE;
			const CURRENT_HASH: &'static str = TERM_L_WHITE;
			const SPEED_BLOCKS: &'static str = TERM_L_YELLOW;
			const SPEED_TXS: &'static str = TERM_L_YELLOW;
			const SPEED_GAS: &'static str = TERM_L_YELLOW;
			const PEERS_ACTIVE: &'static str = TERM_L_GREEN;
			const PEERS_TOTAL: &'static str = TERM_L_GREEN;
			const SYNC_BEST: &'static str = TERM_L_CYAN;
			const SYNC_QUEUED: &'static str = TERM_L_BLUE;
			const SYNC_VERIFIED: &'static str = TERM_L_BLUE;
			const MEM_DB: &'static str = TERM_L_MAGENTA;
			const MEM_CHAIN: &'static str = TERM_L_MAGENTA;
			const MEM_QUEUE: &'static str = TERM_L_MAGENTA;
			const MEM_SYNC: &'static str = TERM_L_MAGENTA;

			println!("{}#{:<7} {}{} {} {}{:3} {}blk/s {}{:3} {}tx/s {}{:4} {}Kgas/s {} {}{:2}{}/{}{:2} {}peers {} {}#{:<7} {}{:4}{}+{}{:4} {}Qed {} {}{:>8} {}db {}{:>8} {}chain {}{:>8} {}queue {}{:>8} {}sync{}",
				CURRENT_NUMBER,
				chain_info.best_block_number,
				CURRENT_HASH,
				chain_info.best_block_hash,
				FRAME,

				SPEED_BLOCKS,
				(report.blocks_imported - last_report.blocks_imported) / dur,
				LABEL_SPEED,
				SPEED_TXS,
				(report.transactions_applied - last_report.transactions_applied) / dur,
				LABEL_SPEED,
				SPEED_GAS,
				((report.gas_processed - last_report.gas_processed) / From::from(dur * 1000)).low_u64(),
				LABEL_SPEED,

				FRAME,

				PEERS_ACTIVE,
				sync_info.num_active_peers,
				LABEL_SLASH,
				PEERS_TOTAL,
				sync_info.num_peers,
				LABEL_PEERS,

				FRAME,

				SYNC_BEST,
				sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number),
				SYNC_QUEUED,
				queue_info.unverified_queue_size,
				LABEL_PLUS,
				SYNC_VERIFIED,
				queue_info.verified_queue_size,
				LABEL_QUEUED,

				FRAME,

				MEM_DB,
				Informant::format_bytes(report.state_db_mem),
				LABEL_MEM,
				MEM_CHAIN,
				Informant::format_bytes(cache_info.total()),
				LABEL_MEM,
				MEM_QUEUE,
				Informant::format_bytes(queue_info.mem_used),
				LABEL_MEM,
				MEM_SYNC,
				Informant::format_bytes(sync_info.mem_used),
				LABEL_MEM,
				TERM_RESET,
			);
		}

		*self.chain_info.write().unwrap().deref_mut() = Some(chain_info);
		*self.cache_info.write().unwrap().deref_mut() = Some(cache_info);
		*write_report.deref_mut() = Some(report);
	}
}

