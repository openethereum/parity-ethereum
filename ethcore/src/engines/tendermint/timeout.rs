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

use std::sync::atomic::{Ordering as AtomicOrdering};
use std::sync::Weak;
use io::{IoContext, IoHandler, TimerToken};
use super::{Tendermint, Step};
use time::{get_time, Duration};
use service::ClientIoMessage;

pub struct TimerHandler {
	engine: Weak<Tendermint>,
}

/// Base timeout of each step in ms.
#[derive(Debug, Clone)]
pub struct TendermintTimeouts {
	propose: Duration,
	prevote: Duartion,
	precommit: Duration,
	commit: Duration
}

impl TendermintTimeouts {
	pub fn for_step(step: Step) -> Duration {
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
		DefaultTimeouts {
			propose: Duration::milliseconds(1000),
			prevote: Duration::milliseconds(1000),
			precommit: Duration::milliseconds(1000),
			commit: Duration::milliseconds(1000)
		}
	}
}

#[derive(Clone)]
pub struct NextStep;

/// Timer token representing the consensus step timeouts.
pub const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

impl IoHandler<NextStep> for TimerHandler {
	fn initialize(&self, io: &IoContext<NextStep>) {
		if let Some(engine) = self.engine.upgrade() {
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.remaining_step_duration().as_millis())
				.unwrap_or_else(|e| warn!(target: "poa", "Failed to start consensus step timer: {}.", e))
		}
	}

	fn timeout(&self, io: &IoContext<NextStep>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				engine.step.fetch_add(1, AtomicOrdering::SeqCst);
				engine.proposed.store(false, AtomicOrdering::SeqCst);
				let next_step = match *engine.step.try_read().unwrap() {
					Step::Propose => Step::Prevote,
					Step::Prevote => Step::Precommit,
					Step::Precommit => Step::Propose,
					Step::Commit => {
						engine.round.fetch_add(1, AtomicOrdering::Relaxed);
						Step::Propose
					},
				};


				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout().as_millis())
					.unwrap_or_else(|e| warn!(target: "poa", "Failed to restart consensus step timer: {}.", e))

				if let Some(ref channel) = *engine.message_channel.lock() {
					match channel.send(ClientIoMessage::UpdateSealing) {
						Ok(_) => trace!(target: "poa", "timeout: UpdateSealing message sent for step {}.", engine.step.),
						Err(err) => trace!(target: "poa", "timeout: Could not send a sealing message {} for step {}.", err, engine.step.load(AtomicOrdering::Relaxed)),
					}
				}

			}
		}
	}

	fn message(&self, io: &IoContext<NextStep>, _net_message: &NextStep) {
		if let Some(engine) = self.engine.upgrade() {
			println!("Message: {:?}", get_time().sec);
			io.clear_timer(ENGINE_TIMEOUT_TOKEN).expect("Failed to restart consensus step timer.");
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Failed to restart consensus step timer.")
		}
	}
}
