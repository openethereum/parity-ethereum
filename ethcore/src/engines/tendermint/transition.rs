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
use io::{IoContext, IoHandler, TimerToken};
use super::{Tendermint, Step};
use time::Duration;

pub struct TransitionHandler {
	pub engine: Weak<Tendermint>,
}

/// Base timeout of each step in ms.
#[derive(Debug, Clone)]
pub struct TendermintTimeouts {
	pub propose: Duration,
	pub prevote: Duration,
	pub precommit: Duration,
	pub commit: Duration
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
			propose: Duration::milliseconds(1000),
			prevote: Duration::milliseconds(1000),
			precommit: Duration::milliseconds(1000),
			commit: Duration::milliseconds(1000)
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
		if let Some(engine) = self.engine.upgrade() {
			set_timeout(io, engine.our_params.timeouts.propose)
		}
	}

	fn timeout(&self, io: &IoContext<Step>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				let next_step = match *engine.step.read() {
					Step::Propose => {
						set_timeout(io, engine.our_params.timeouts.prevote);
						Some(Step::Prevote)
					},
					Step::Prevote if engine.has_enough_any_votes() => {
						set_timeout(io, engine.our_params.timeouts.precommit);
						Some(Step::Precommit)
					},
					Step::Precommit if engine.has_enough_any_votes() => {
						set_timeout(io, engine.our_params.timeouts.propose);
						engine.increment_round(1);
						Some(Step::Propose)
					},
					Step::Commit => {
						set_timeout(io, engine.our_params.timeouts.propose);
						engine.reset_round();
						Some(Step::Propose)
					},
					_ => None,
				};

				if let Some(step) = next_step {
					engine.to_step(step)
				}
			}
		}
	}

	fn message(&self, io: &IoContext<Step>, next_step: &Step) {
		if let Some(engine) = self.engine.upgrade() {
			if let Err(io_err) = io.clear_timer(ENGINE_TIMEOUT_TOKEN) {
				warn!(target: "poa", "Could not remove consensus timer {}.", io_err)
			}
			match *next_step {
				Step::Propose => set_timeout(io, engine.our_params.timeouts.propose),
				Step::Prevote => set_timeout(io, engine.our_params.timeouts.prevote),
				Step::Precommit => set_timeout(io, engine.our_params.timeouts.precommit),
				Step::Commit => set_timeout(io, engine.our_params.timeouts.commit),
			};
			engine.to_step(*next_step);
		}
	}
}
