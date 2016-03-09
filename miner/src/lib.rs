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

#![warn(missing_docs)]
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]

#[macro_use]
extern crate log;
#[macro_use]
extern crate ethcore_util as util;
extern crate ethcore;
extern crate env_logger;
extern crate rayon;

mod miner;
mod transaction_queue;

use std::ops::*;
use std::sync::*;
pub use miner::{Miner, MinerService};


pub struct EthMiner {
	miner: Miner,
}

impl EthMiner {
	/// Creates and register protocol with the network service
	pub fn new() -> Arc<EthMiner> {
		Arc::new(EthMiner {
			miner: Miner::new(),
		})
	}
}

impl Deref for EthMiner {
	type Target = Miner;

	fn deref(&self) -> &Self::Target {
		&self.miner
	}
}
