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

extern crate ansi_term;
use self::ansi_term::Colour::{White, Yellow, Green, Cyan, Blue, Purple};

use std::time::{Instant, Duration};
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
	last_tick: RwLock<Instant>,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
			last_tick: RwLock::new(Instant::now()),
		}
	}
}

trait MillisecondDuration {
	fn as_milliseconds(&self) -> u64;
}

impl MillisecondDuration for Duration {
	fn as_milliseconds(&self) -> u64 {
		self.as_secs() * 1000 + self.subsec_nanos() as u64 / 1000000
	}
}

impl Informant {
	fn format_bytes(b: usize) -> String {
		match binary_prefix(b as f64) {
			Standalone(bytes)   => format!("{} bytes", bytes),
			Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
		}
	}

	pub fn tick(&self, client: &Client, maybe_sync: Option<&EthSync>) {
		let elapsed = self.last_tick.read().unwrap().elapsed();
		if elapsed < Duration::from_secs(5) {
			return;
		}

		*self.last_tick.write().unwrap() = Instant::now();

		let chain_info = client.chain_info();
		let queue_info = client.queue_info();
		let cache_info = client.blockchain_cache_info();

		let mut write_report = self.report.write().unwrap();
		let report = client.report();

		if let (_, _, &Some(ref last_report)) = (
			self.chain_info.read().unwrap().deref(),
			self.cache_info.read().unwrap().deref(),
			write_report.deref()
		) {
			println!("#{} {}   {} blk/s {} tx/s {} Kgas/s   {}{}+{} Qed   {} db {} chain {} queue{}",
				White.bold().paint(format!("{:<7}", chain_info.best_block_number)),
				White.bold().paint(format!("{}", chain_info.best_block_hash)),

				Yellow.bold().paint(format!("{:3}", ((report.blocks_imported - last_report.blocks_imported) * 1000) as u64 / elapsed.as_milliseconds())),
				Yellow.bold().paint(format!("{:3}", ((report.transactions_applied - last_report.transactions_applied) * 1000) as u64 / elapsed.as_milliseconds())),
				Yellow.bold().paint(format!("{:4}", ((report.gas_processed - last_report.gas_processed) * From::from(1000000 / elapsed.as_milliseconds())).low_u64())),

				match maybe_sync {
					Some(sync) => {
						let sync_info = sync.status();
						format!("{}/{} peers   #{} ",
							Green.bold().paint(format!("{:2}", sync_info.num_active_peers)),
							Green.bold().paint(format!("{:2}", sync_info.num_peers)),
							Cyan.bold().paint(format!("{:<7}", sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number))),
						)
					}
					None => String::new()
				},

				Blue.bold().paint(format!("{:4}", queue_info.unverified_queue_size)),
				Blue.bold().paint(format!("{:4}", queue_info.verified_queue_size)),

				Purple.bold().paint(format!("{:>8}", Informant::format_bytes(report.state_db_mem))),
				Purple.bold().paint(format!("{:>8}", Informant::format_bytes(cache_info.total()))),
				Purple.bold().paint(format!("{:>8}", Informant::format_bytes(queue_info.mem_used))),
				match maybe_sync {
					Some(sync) => {
						let sync_info = sync.status();
						format!(" {} sync", Purple.bold().paint(format!("{:>8}", Informant::format_bytes(sync_info.mem_used))))
					}
					None => String::new()
				},
			);
		}

		*self.chain_info.write().unwrap().deref_mut() = Some(chain_info);
		*self.cache_info.write().unwrap().deref_mut() = Some(cache_info);
		*write_report.deref_mut() = Some(report);
	}
}

