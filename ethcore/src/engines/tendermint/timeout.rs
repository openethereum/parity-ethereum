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
use time::get_time;

pub struct TimerHandler {
	engine: Weak<Tendermint>,
}

impl TimerHandler {
	pub fn new(engine: Weak<Tendermint>) -> Self {
		TimerHandler { engine: engine }
	}
}

/// Base timeout of each step in ms.
#[derive(Debug, Clone)]
pub struct DefaultTimeouts {
	pub propose: Ms,
	pub prevote: Ms,
	pub precommit: Ms,
	pub commit: Ms
}

impl Default for DefaultTimeouts {
	fn default() -> Self {
		DefaultTimeouts {
			propose: 1000,
			prevote: 1000,
			precommit: 1000,
			commit: 1000
		}
	}
}

pub type Ms = usize;

#[derive(Clone)]
pub struct NextStep;

/// Timer token representing the consensus step timeouts.
pub const ENGINE_TIMEOUT_TOKEN: TimerToken = 0;

impl IoHandler<NextStep> for TimerHandler {
	fn initialize(&self, io: &IoContext<NextStep>) {
		if let Some(engine) = self.engine.upgrade() {
			io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Error registering engine timeout");
		}
	}

	fn timeout(&self, io: &IoContext<NextStep>, timer: TimerToken) {
		if timer == ENGINE_TIMEOUT_TOKEN {
			if let Some(engine) = self.engine.upgrade() {
				println!("Timeout: {:?}", get_time());
				// Can you release entering a clause?
				let next_step = match *engine.s.try_read().unwrap() {
					Step::Propose => Step::Propose,
					Step::Prevote(_) => Step::Propose,
					Step::Precommit(_, _) => Step::Propose,
					Step::Commit(_, _) => {
						engine.r.fetch_add(1, AtomicOrdering::Relaxed);
						Step::Propose
					},
				};
				match next_step {
					Step::Propose => engine.to_propose(),
					_ => (),
				}
				io.register_timer_once(ENGINE_TIMEOUT_TOKEN, engine.next_timeout()).expect("Failed to restart consensus step timer.")
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
