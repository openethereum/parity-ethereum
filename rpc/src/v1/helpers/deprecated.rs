// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Deprecation notice for RPC methods.
//!
//! Displays a warning but avoids spamming the log.

use std::{
	collections::HashMap,
	time::{Duration, Instant},
};

use parking_lot::RwLock;

/// Deprecation messages
pub mod msgs {
	pub const ACCOUNTS: Option<&str> = Some("Account management is being phased out see #9997 for alternatives.");
}

type MethodName = &'static str;

const PRINT_INTERVAL: Duration = Duration::from_secs(60);

/// Displays a deprecation notice without spamming the log.
pub struct DeprecationNotice<T = fn() -> Instant> {
	now: T,
	next_warning_at: RwLock<HashMap<String, Instant>>,
	printer: Box<Fn(MethodName, Option<&str>) + Send + Sync>,
}

impl Default for DeprecationNotice {
	fn default() -> Self {
		Self::new(Instant::now, |method, more| {
			let more = more.map(|x| format!(": {}", x)).unwrap_or_else(|| ".".into());
			warn!(target: "rpc", "{} is deprecated and will be removed in future versions{}", method, more);
		})
	}
}

impl<N: Fn() -> Instant> DeprecationNotice<N> {
	/// Create new deprecation notice printer with custom display and interval.
	pub fn new<T>(now: N, printer: T) -> Self where
		T: Fn(MethodName, Option<&str>) + Send + Sync + 'static,
	{
		DeprecationNotice {
			now,
			next_warning_at: Default::default(),
			printer: Box::new(printer),
		}
	}

	/// Print deprecation notice for given method and with some additional details (explanations).
	pub fn print<'a, T: Into<Option<&'a str>>>(&self, method: MethodName, details: T) {
		let now = (self.now)();
		match self.next_warning_at.read().get(method) {
			Some(next) if *next > now => return,
			_ => {},
		}

		self.next_warning_at.write().insert(method.to_owned(), now + PRINT_INTERVAL);
		(self.printer)(method, details.into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::sync::Arc;

	#[test]
	fn should_throttle_printing() {
		let saved = Arc::new(RwLock::new(None));
		let s = saved.clone();
		let printer = move |method: MethodName, more: Option<&str>| {
			*s.write() = Some((method, more.map(|s| s.to_owned())));
		};

		let now = Arc::new(RwLock::new(Instant::now()));
		let n = now.clone();
		let get_now = || n.read().clone();
		let notice = DeprecationNotice::new(get_now, printer);

		let details = Some("See issue #123456");
		notice.print("eth_test", details.clone());
		// printer shouldn't be called
		notice.print("eth_test", None);
		assert_eq!(saved.read().clone().unwrap(), ("eth_test", details.as_ref().map(|x| x.to_string())));
		// but calling a different method is fine
		notice.print("eth_test2", None);
		assert_eq!(saved.read().clone().unwrap(), ("eth_test2", None));

		// wait and call again
		*now.write() = Instant::now() + PRINT_INTERVAL;
		notice.print("eth_test", None);
		assert_eq!(saved.read().clone().unwrap(), ("eth_test", None));
	}
}
