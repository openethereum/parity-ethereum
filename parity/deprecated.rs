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

use std::fmt;
use cli::Args;

#[derive(Debug, PartialEq)]
pub enum Deprecated {
	DoesNothing(&'static str),
	Replaced(&'static str, &'static str),
	Removed(&'static str),
}

impl fmt::Display for Deprecated {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Deprecated::DoesNothing(s) => write!(f, "Option '{}' does nothing. It's on by default.", s),
			Deprecated::Replaced(old, new) => write!(f, "Option '{}' is deprecated. Please use '{}' instead.", old, new),
			Deprecated::Removed(s) => write!(f, "Option '{}' has been removed and is no longer supported.", s)
		}
	}
}

pub fn find_deprecated(args: &Args) -> Vec<Deprecated> {
	let mut result = vec![];

	// Removed in 1.6 or before.

	if args.flag_warp {
		result.push(Deprecated::DoesNothing("--warp"));
	}

	if args.flag_jsonrpc {
		result.push(Deprecated::DoesNothing("--jsonrpc"));
	}

	if args.flag_rpc {
		result.push(Deprecated::DoesNothing("--rpc"));
	}

	if args.flag_jsonrpc_off {
		result.push(Deprecated::Replaced("--jsonrpc-off", "--no-jsonrpc"));
	}

	if args.flag_webapp {
		result.push(Deprecated::DoesNothing("--webapp"));
	}

	if args.flag_dapps_off {
		result.push(Deprecated::Replaced("--dapps-off", "--no-dapps"));
	}

	if args.flag_ipcdisable {
		result.push(Deprecated::Replaced("--ipcdisable", "--no-ipc"));
	}

	if args.flag_ipc_off {
		result.push(Deprecated::Replaced("--ipc-off", "--no-ipc"));
	}

	if args.arg_etherbase.is_some() {
		result.push(Deprecated::Replaced("--etherbase", "--author"));
	}

	if args.arg_extradata.is_some() {
		result.push(Deprecated::Replaced("--extradata", "--extra-data"));
	}

	if args.flag_testnet {
		result.push(Deprecated::Replaced("--testnet", "--chain testnet"));
	}

	if args.flag_nodiscover {
		result.push(Deprecated::Replaced("--nodiscover", "--no-discovery"));
	}

	if args.arg_datadir.is_some() {
		result.push(Deprecated::Replaced("--datadir", "--base-path"));
	}

	if args.arg_networkid.is_some() {
		result.push(Deprecated::Replaced("--networkid", "--network-id"));
	}

	if args.arg_peers.is_some() {
		result.push(Deprecated::Replaced("--peers", "--min-peers"));
	}

	if args.arg_nodekey.is_some() {
		result.push(Deprecated::Replaced("--nodekey", "--node-key"));
	}

	if args.arg_rpcaddr.is_some() {
		result.push(Deprecated::Replaced("--rpcaddr", "--jsonrpc-interface"));
	}

	if args.arg_rpcport.is_some() {
		result.push(Deprecated::Replaced("--rpcport", "--jsonrpc-port"));
	}

	if args.arg_rpcapi.is_some() {
		result.push(Deprecated::Replaced("--rpcapi", "--jsonrpc-api"));
	}

	if args.arg_rpccorsdomain.is_some() {
		result.push(Deprecated::Replaced("--rpccorsdomain", "--jsonrpc-cors"));
	}

	if args.arg_ipcapi.is_some() {
		result.push(Deprecated::Replaced("--ipcapi", "--ipc-apis"));
	}

	if args.arg_ipcpath.is_some() {
		result.push(Deprecated::Replaced("--ipcpath", "--ipc-path"));
	}

	if args.arg_gasprice.is_some() {
		result.push(Deprecated::Replaced("--gasprice", "--min-gas-price"));
	}

	if args.arg_cache.is_some() {
		result.push(Deprecated::Replaced("--cache", "--cache-size"));
	}

	// Removed in 1.7.

	if args.arg_dapps_port.is_some() {
		result.push(Deprecated::Removed("--dapps-port"));
	}

	if args.arg_dapps_interface.is_some() {
		result.push(Deprecated::Removed("--dapps-interface"));
	}

	if args.arg_dapps_hosts.is_some() {
		result.push(Deprecated::Removed("--dapps-hosts"));
	}

	if args.arg_dapps_cors.is_some() {
		result.push(Deprecated::Removed("--dapps-cors"));
	}

	if args.arg_dapps_user.is_some() {
		result.push(Deprecated::Removed("--dapps-user"));
	}

	if args.arg_dapps_pass.is_some() {
		result.push(Deprecated::Removed("--dapps-pass"));
	}

	if args.flag_dapps_apis_all {
		result.push(Deprecated::Replaced("--dapps-apis-all", "--jsonrpc-apis"));
	}

	// Removed in 1.11.

	if args.flag_public_node {
		result.push(Deprecated::Removed("--public-node"));
	}

	if args.flag_force_ui {
		result.push(Deprecated::Removed("--force-ui"));
	}

	if args.flag_no_ui {
		result.push(Deprecated::Removed("--no-ui"));
	}

	if args.flag_ui_no_validation {
		result.push(Deprecated::Removed("--ui-no-validation"));
	}

	if args.arg_ui_interface.is_some() {
		result.push(Deprecated::Removed("--ui-interface"));
	}

	if args.arg_ui_hosts.is_some() {
		result.push(Deprecated::Removed("--ui-hosts"));
	}

	if args.arg_ui_port.is_some() {
		result.push(Deprecated::Removed("--ui-port"));
	}

	if args.arg_tx_queue_ban_count.is_some() {
		result.push(Deprecated::Removed("--tx-queue-ban-count"));
	}

	if args.arg_tx_queue_ban_time.is_some() {
		result.push(Deprecated::Removed("--tx-queue-ban-time"));
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
			args.flag_warp = true;
			args.flag_jsonrpc = true;
			args.flag_rpc = true;
			args.flag_jsonrpc_off = true;
			args.flag_webapp = true;
			args.flag_dapps_off = true;
			args.flag_ipcdisable = true;
			args.flag_ipc_off = true;
			args.arg_etherbase = Some(Default::default());
			args.arg_extradata = Some(Default::default());
			args.arg_dapps_port = Some(Default::default());
			args.arg_dapps_interface = Some(Default::default());
			args.arg_dapps_hosts = Some(Default::default());
			args.arg_dapps_cors = Some(Default::default());
			args.arg_dapps_user = Some(Default::default());
			args.arg_dapps_pass = Some(Default::default());
			args.flag_dapps_apis_all = true;
			args
		}), vec![
			Deprecated::DoesNothing("--warp"),
			Deprecated::DoesNothing("--jsonrpc"),
			Deprecated::DoesNothing("--rpc"),
			Deprecated::Replaced("--jsonrpc-off", "--no-jsonrpc"),
			Deprecated::DoesNothing("--webapp"),
			Deprecated::Replaced("--dapps-off", "--no-dapps"),
			Deprecated::Replaced("--ipcdisable", "--no-ipc"),
			Deprecated::Replaced("--ipc-off", "--no-ipc"),
			Deprecated::Replaced("--etherbase", "--author"),
			Deprecated::Replaced("--extradata", "--extra-data"),
			Deprecated::Removed("--dapps-port"),
			Deprecated::Removed("--dapps-interface"),
			Deprecated::Removed("--dapps-hosts"),
			Deprecated::Removed("--dapps-cors"),
			Deprecated::Removed("--dapps-user"),
			Deprecated::Removed("--dapps-pass"),
			Deprecated::Replaced("--dapps-apis-all", "--jsonrpc-apis"),
		]);
	}
}
