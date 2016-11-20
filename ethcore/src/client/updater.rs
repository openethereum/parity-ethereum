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

use std::sync::Weak;
use util::misc::code_hash;
use util::{Address, H160};
use client::operations::Operations;
use client::client::Client;

pub struct Updater {
	operations: Operations,
}

fn platform() -> &'static str {
	"linux_x64"
}

impl Updater {
	pub fn new(client: Weak<Client>, operations: Address) -> Self {
		Updater {
			operations: Operations::new(operations, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))),
		}
	}

	pub fn tick(&mut self) {
		(|| -> Result<(), String> {
			let code_hash = H160::from("0x080ec8043f41e25ee8aa4ee6112906ac6d82ea74").into();//code_hash().into();
			let client = "parity";

			let (fork, track, semver) = self.operations.find_release(client, &code_hash)?;
			let track_name = match track { 1 => "stable", 2 => "beta", 3 => "nightly", _ => "unknown" };
			info!(target: "updater", "Current release ({}) is {}.{}.{}-{} and latest fork it supports is at block #{}", H160::from(code_hash), semver >> 16, (semver >> 8) & 0xff, semver & 0xff, track_name, fork);

			let latest_fork = self.operations.latest_fork()?;
			info!(target: "updater", "Latest fork is at block #{}", latest_fork);

			let latest = self.operations.latest_in_track(client, track)?;
			let (fork, _, semver) = self.operations.find_release(client, &latest)?;
			info!(target: "updater", "Latest release in our track is {}.{}.{}-{} ({:?}); supports fork at block #{}", semver >> 16, (semver >> 8) & 0xff, semver & 0xff, track_name, H160::from(latest), fork);

			let exe_hash = self.operations.find_checksum(client, &latest, platform())?;
			info!(target: "updater", "Latest release's binary on {} is {}", platform(), exe_hash);
			Ok(())
		})().unwrap_or_else(|e| warn!("{}", e));
	}
}
