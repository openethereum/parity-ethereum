// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Client related types.

use std::{
	fmt::{Display, Formatter, Error as FmtError},
	ops,
	cmp,
	time::Duration,
};
use ethereum_types::U256;
use crate::header::Header;

/// Operating mode for the client.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Mode {
	/// Always on.
	Active,
	/// Goes offline after client is inactive for some (given) time, but
	/// comes back online after a while of inactivity.
	Passive(Duration, Duration),
	/// Goes offline after client is inactive for some (given) time and
	/// stays inactive.
	Dark(Duration),
	/// Always off.
	Off,
}

impl Display for Mode {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Mode::Active => write!(f, "active"),
			Mode::Passive(..) => write!(f, "passive"),
			Mode::Dark(..) => write!(f, "dark"),
			Mode::Off => write!(f, "offline"),
		}
	}
}

/// Report on the status of a client.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct ClientReport {
	/// How many blocks have been imported so far.
	pub blocks_imported: usize,
	/// How many transactions have been applied so far.
	pub transactions_applied: usize,
	/// How much gas has been processed so far.
	pub gas_processed: U256,
	/// Memory used by state DB
	pub state_db_mem: usize,
}

impl ClientReport {
	/// Alter internal reporting to reflect the additional `block` has been processed.
	pub fn accrue_block(&mut self, header: &Header, transactions: usize) {
		self.blocks_imported += 1;
		self.transactions_applied += transactions;
		self.gas_processed = self.gas_processed + *header.gas_used();
	}
}

impl<'a> ops::Sub<&'a ClientReport> for ClientReport {
	type Output = Self;

	fn sub(mut self, other: &'a ClientReport) -> Self {
		let higher_mem = cmp::max(self.state_db_mem, other.state_db_mem);
		let lower_mem = cmp::min(self.state_db_mem, other.state_db_mem);

		self.blocks_imported -= other.blocks_imported;
		self.transactions_applied -= other.transactions_applied;
		self.gas_processed = self.gas_processed - other.gas_processed;
		self.state_db_mem = higher_mem - lower_mem;

		self
	}
}

