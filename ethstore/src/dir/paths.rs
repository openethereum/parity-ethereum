// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Common tools paths.

use std::env;
use std::path::PathBuf;

fn home() -> PathBuf {
	env::home_dir().expect("Failed to get home dir")
}

/// Geth path
pub fn geth(testnet: bool) -> PathBuf {
	let mut base = geth_base();
	if testnet {
		base.push("testnet");
	}
	base.push("keystore");
	base
}

/// Parity path for specific chain
pub fn parity(chain: &str) -> PathBuf {
	let mut base = parity_base();
	base.push(chain);
	base
}

#[cfg(target_os = "macos")]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push("Library");
	home.push("Application Support");
	home.push("io.parity.ethereum");
	home.push("keys");
	home
}

#[cfg(windows)]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push("AppData");
	home.push("Roaming");
	home.push("Parity");
	home.push("Ethereum");
	home.push("keys");
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
fn parity_base() -> PathBuf {
	let mut home = home();
	home.push(".local");
	home.push("share");
	home.push("io.parity.ethereum");
	home.push("keys");
	home
}

#[cfg(target_os = "macos")]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push("Library");
	home.push("Ethereum");
	home
}

#[cfg(windows)]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push("AppData");
	home.push("Roaming");
	home.push("Ethereum");
	home
}

#[cfg(not(any(target_os = "macos", windows)))]
fn geth_base() -> PathBuf {
	let mut home = home();
	home.push(".ethereum");
	home
}
