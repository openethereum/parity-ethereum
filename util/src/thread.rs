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

//! Thread management helpers

use libc::{c_int, pthread_self, pthread_t};

#[repr(C)]
struct sched_param {
    priority: c_int,
    padding: c_int,
}

extern {
	fn setpriority(which: c_int, who: c_int, prio: c_int) -> c_int;
	fn pthread_setschedparam(thread: pthread_t, policy: c_int, param: *const sched_param) -> c_int;
}
const PRIO_DARWIN_THREAD: c_int = 3;
const PRIO_DARWIN_BG: c_int = 0x1000;
const SCHED_RR: c_int = 2;

/// Lower thread priority and put it into background mode
#[cfg(target_os="macos")]
pub fn lower_thread_priority() {
	let sp = sched_param { priority: 0, padding: 0 };
	if unsafe { pthread_setschedparam(pthread_self(), SCHED_RR, &sp) } == -1 {
		trace!("Could not decrease thread piority");
	}
	//unsafe { setpriority(PRIO_DARWIN_THREAD, 0, PRIO_DARWIN_BG); }
}
