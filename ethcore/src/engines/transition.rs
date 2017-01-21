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

//! Tendermint timeout handling.

use std::sync::Weak;
use time::Duration;
use io::{IoContext, IoHandler, TimerToken};
use super::{Tendermint, Step};
use engines::Engine;

pub struct TransitionHandler {
	engine: Weak<Tendermint>,
	timeouts: TendermintTimeouts,
}

impl TransitionHandler {
	pub fn new(engine: Weak<Tendermint>, timeouts: TendermintTimeouts) -> Self {
		TransitionHandler {
			engine: engine,
			timeouts: timeouts,
		}
	}
}

/// Base timeout of each step in ms.
#[derive(Debug, Clone)]
pub struct TendermintTimeouts {
	pub propose: Duration,
	pub prevote: Duration,
	pub precommit: Duration,
	pub commit: Duration,
}

impl TendermintTimeouts {
	pub fn for_step(&self, step: Step) -> Duration {
		match step {
			Step::Propose => self.propose,
			Step::Prevote => self.prevote,
			Step::Precommit => self.precommit,
			Step::Commit => self.commit,
		}
	}
}

impl Default for TendermintTimeouts {
	fn default() -> Self {
		TendermintTimeouts {
			propose: Duration::milliseconds(10000),
			prevote: Duration::milliseconds(10000),
			precommit: Duration::milliseconds(10000),
			commit: Duration::milliseconds(10000),
		}
	}
}

/// Timer token representing the consensus step timeouts.
pub const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

fn set_timeout(io: &IoContext<Step>, timeout: Duration) {
	io.register_timer_once(ENGINE_TIMEOUT_TOKEN, timeout.num_milliseconds() as u64)
		.unwrap_or_else(|e| warn!(target: "poa", "Failed to set consensus step timeout: {}.", e))
}

impl IoHandler<Step> for TransitionHandler {
	fn initialize(&self, io: &IoContext<Step>) {
		set_timeout(io, self.timeouts.propose)
	}

	fn timeout(&self, _io: &IoContext<Step>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				engine.step();
			}
		}
	}

	fn message(&self, io: &IoContext<Step>, next_step: &Step) {
		if let Err(io_err) = io.clear_timer(ENGINE_TIMEOUT_TOKEN) {
			warn!(target: "poa", "Could not remove consensus timer {}.", io_err)
		}
		match *next_step {
			Step::Propose => set_timeout(io, self.timeouts.propose),
			Step::Prevote => set_timeout(io, self.timeouts.prevote),
			Step::Precommit => set_timeout(io, self.timeouts.precommit),
			Step::Commit => set_timeout(io, self.timeouts.commit),
		};
	}
}
