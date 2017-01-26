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

use std::fmt;
use cli::Args;

#[derive(Debug, PartialEq)]
pub enum Deprecated {
	DoesNothing(&'static str),
	Replaced(&'static str, &'static str),
}

impl fmt::Display for Deprecated {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Deprecated::DoesNothing(s) => write!(f, "Option '{}' does nothing. It's on by default", s),
			Deprecated::Replaced(old, new) => write!(f, "Option '{}' is deprecated. Please use '{}' instead", old, new),
		}
	}
}

impl Deprecated {
	fn jsonrpc() -> Self {
		Deprecated::DoesNothing("--jsonrpc")
	}

	fn rpc() -> Self {
		Deprecated::DoesNothing("--rpc")
	}

	fn jsonrpc_off() -> Self {
		Deprecated::Replaced("--jsonrpc-off", "--no-jsonrpc")
	}

	fn webapp() -> Self {
		Deprecated::DoesNothing("--webapp")
	}

	fn dapps_off() -> Self {
		Deprecated::Replaced("--dapps-off", "--no-dapps")
	}

	fn ipcdisable() -> Self {
		Deprecated::Replaced("--ipcdisable", "--no-ipc")
	}

	fn ipc_off() -> Self {
		Deprecated::Replaced("--ipc-off", "--no-ipc")
	}

	fn etherbase() -> Self {
		Deprecated::Replaced("--etherbase", "--author")
	}

	fn extradata() -> Self {
		Deprecated::Replaced("--extradata", "--extra-data")
	}
}

pub fn find_deprecated(args: &Args) -> Vec<Deprecated> {
	let mut result = vec![];

	if args.flag_jsonrpc {
		result.push(Deprecated::jsonrpc());
	}

	if args.flag_rpc {
		result.push(Deprecated::rpc());
	}

	if args.flag_jsonrpc_off {
		result.push(Deprecated::jsonrpc_off());
	}

	if args.flag_webapp {
		result.push(Deprecated::webapp())
	}

	if args.flag_dapps_off {
		result.push(Deprecated::dapps_off());
	}

	if args.flag_ipcdisable {
		result.push(Deprecated::ipcdisable());
	}

	if args.flag_ipc_off {
		result.push(Deprecated::ipc_off());
	}

	if args.flag_etherbase.is_some() {
		result.push(Deprecated::etherbase());
	}

	if args.flag_extradata.is_some() {
		result.push(Deprecated::extradata());
	}

	result
}

#[cfg(test)]
mod tests {
	use cli::Args;
	use super::{Deprecated, find_deprecated};

	#[test]
	fn test_find_deprecated() {
		assert_eq!(find_deprecated(&Args::default()), vec![]);
		assert_eq!(find_deprecated(&{
			let mut args = Args::default();
			args.flag_jsonrpc = true;
			args.flag_rpc = true;
			args.flag_jsonrpc_off = true;
			args.flag_webapp = true;
			args.flag_dapps_off = true;
			args.flag_ipcdisable = true;
			args.flag_ipc_off = true;
			args.flag_etherbase = Some(Default::default());
			args.flag_extradata = Some(Default::default());
			args
		}), vec![
			Deprecated::jsonrpc(),
			Deprecated::rpc(),
			Deprecated::jsonrpc_off(),
			Deprecated::webapp(),
			Deprecated::dapps_off(),
			Deprecated::ipcdisable(),
			Deprecated::ipc_off(),
			Deprecated::etherbase(),
			Deprecated::extradata(),
		]);
	}
}

