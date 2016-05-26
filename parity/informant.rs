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
use self::ansi_term::Style;

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
	with_color: bool,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
			last_tick: RwLock::new(Instant::now()),
			with_color: true,
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
	/// Make a new instance potentially `with_color` output.
	pub fn new(with_color: bool) -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
			last_tick: RwLock::new(Instant::now()),
			with_color: with_color,
		}
	}

	fn format_bytes(b: usize) -> String {
		match binary_prefix(b as f64) {
			Standalone(bytes)   => format!("{} bytes", bytes),
			Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
		}
	}

	#[cfg_attr(feature="dev", allow(match_bool))]
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

		let paint = |c: Style, t: String| match self.with_color {
			true => format!("{}", c.paint(t)),
			false => t,
		};

		if let (_, _, &Some(ref last_report)) = (
			self.chain_info.read().unwrap().deref(),
			self.cache_info.read().unwrap().deref(),
			write_report.deref()
		) {
			println!("{} {}   {} blk/s {} tx/s {} Mgas/s   {}{}+{} Qed   {} db {} chain {} queue{}",
				paint(White.bold(), format!("{:>8}", format!("#{}", chain_info.best_block_number))),
				paint(White.bold(), format!("{}", chain_info.best_block_hash)),

				paint(Yellow.bold(), format!("{:4}", ((report.blocks_imported - last_report.blocks_imported) * 1000) as u64 / elapsed.as_milliseconds())),
				paint(Yellow.bold(), format!("{:4}", ((report.transactions_applied - last_report.transactions_applied) * 1000) as u64 / elapsed.as_milliseconds())),
				paint(Yellow.bold(), format!("{:3}", ((report.gas_processed - last_report.gas_processed) / From::from(elapsed.as_milliseconds() * 1000)).low_u64())),

				match maybe_sync {
					Some(sync) => {
						let sync_info = sync.status();
						format!("{}/{} peers   {} ",
							paint(Green.bold(), format!("{:2}", sync_info.num_active_peers)),
							paint(Green.bold(), format!("{:2}", sync_info.num_peers)),
							paint(Cyan.bold(), format!("{:>8}", format!("#{}", sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number)))),
						)
					}
					None => String::new()
				},

				paint(Blue.bold(), format!("{:5}", queue_info.unverified_queue_size)),
				paint(Blue.bold(), format!("{:5}", queue_info.verified_queue_size)),

				paint(Purple.bold(), format!("{:>8}", Informant::format_bytes(report.state_db_mem))),
				paint(Purple.bold(), format!("{:>8}", Informant::format_bytes(cache_info.total()))),
				paint(Purple.bold(), format!("{:>8}", Informant::format_bytes(queue_info.mem_used))),
				match maybe_sync {
					Some(sync) => {
						let sync_info = sync.status();
						format!(" {} sync", paint(Purple.bold(), format!("{:>8}", Informant::format_bytes(sync_info.mem_used))))
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

