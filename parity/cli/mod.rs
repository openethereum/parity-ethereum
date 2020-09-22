// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
mod usage;
mod presets;

use super::helpers;
use std::collections::HashSet;

usage! {
    {
        // CLI subcommands
        // Subcommands must start with cmd_ and have '_' in place of '-'
        // Sub-subcommands must start with the name of the subcommand
        // Arguments must start with arg_
        // Flags must start with flag_

        CMD cmd_daemon
        {
            "Use OpenEthereum as a daemon",

            ARG arg_daemon_pid_file: (Option<String>) = None,
            "<PID-FILE>",
            "Path to the pid file",
        }

        CMD cmd_account
        {
            "Manage accounts",

            CMD cmd_account_new {
                "Create a new account (and its associated key) for the given --chain (default: mainnet)",
            }

            CMD cmd_account_list {
                "List existing accounts of the given --chain (default: mainnet)",
            }

            CMD cmd_account_import
            {
                "Import accounts from JSON UTC keystore files to the specified --chain (default mainnet)",

                ARG arg_account_import_path : (Option<Vec<String>>) = None,
                "<PATH>...",
                "Path to the accounts",
            }
        }

        CMD cmd_wallet
        {
            "Manage wallet",

            CMD cmd_wallet_import
            {
                "Import wallet into the given --chain (default: mainnet)",

                ARG arg_wallet_import_path: (Option<String>) = None,
                "<PATH>",
                "Path to the wallet",
            }
        }

        CMD cmd_import
        {
            "Import blockchain data from a file to the given --chain database (default: mainnet)",

            ARG arg_import_format: (Option<String>) = None,
            "--format=[FORMAT]",
            "Import in a given format. FORMAT must be either 'hex' or 'binary'. (default: auto)",

            ARG arg_import_file: (Option<String>) = None,
            "[FILE]",
            "Path to the file to import from",
        }

        CMD cmd_export
        {
            "Export blockchain",

            CMD cmd_export_blocks
            {
                "Export the blockchain blocks from the given --chain database (default: mainnet) into a file. This command requires the chain to be synced with --fat-db on.",

                ARG arg_export_blocks_format: (Option<String>) = None,
                "--format=[FORMAT]",
                "Export in a given format. FORMAT must be either 'hex' or 'binary'. (default: binary)",

                ARG arg_export_blocks_from: (String) = "1",
                "--from=[BLOCK]",
                "Export from block BLOCK, which may be an index or hash.",

                ARG arg_export_blocks_to: (String) = "latest",
                "--to=[BLOCK]",
                "Export to (including) block BLOCK, which may be an index, hash or latest.",

                ARG arg_export_blocks_file: (Option<String>) = None,
                "[FILE]",
                "Path to the exported file",
            }

            CMD cmd_export_state
            {
                "Export the blockchain state from the given --chain (default: mainnet) into a file. This command requires the chain to be synced with --fat-db on.",

                FLAG flag_export_state_no_storage: (bool) = false,
                "--no-storage",
                "Don't export account storage.",

                FLAG flag_export_state_no_code: (bool) = false,
                "--no-code",
                "Don't export account code.",

                ARG arg_export_state_min_balance: (Option<String>) = None,
                "--min-balance=[WEI]",
                "Don't export accounts with balance less than specified.",

                ARG arg_export_state_max_balance: (Option<String>) = None,
                "--max-balance=[WEI]",
                "Don't export accounts with balance greater than specified.",

                ARG arg_export_state_at: (String) = "latest",
                "--at=[BLOCK]",
                "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive",

                ARG arg_export_state_format: (Option<String>) = None,
                "--format=[FORMAT]",
                "Export in a given format. FORMAT must be either 'hex' or 'binary'. (default: binary)",

                ARG arg_export_state_file: (Option<String>) = None,
                "[FILE]",
                "Path to the exported file",
            }
        }

        CMD cmd_signer
        {
            "Manage signer",

            CMD cmd_signer_new_token {
                "Generate a new signer-authentication token for the given --chain (default: mainnet)",
            }

            CMD cmd_signer_list {
                "List the signer-authentication tokens from given --chain (default: mainnet)",
            }

            CMD cmd_signer_sign
            {
                "Sign",

                ARG arg_signer_sign_id: (Option<usize>) = None,
                "[ID]",
                "ID",
            }

            CMD cmd_signer_reject
            {
                "Reject",

                ARG arg_signer_reject_id: (Option<usize>) = None,
                "<ID>",
                "ID",
            }
        }

        CMD cmd_snapshot
        {
            "Make a snapshot of the database of the given --chain (default: mainnet)",

            ARG arg_snapshot_at: (String) = "latest",
            "--at=[BLOCK]",
            "Take a snapshot at the given block, which may be an index, hash, or latest. Note that taking snapshots at non-recent blocks will only work with --pruning archive",

            ARG arg_snapshot_file: (Option<String>) = None,
            "<FILE>",
            "Path to the file to export to",
        }

        CMD cmd_restore
        {
            "Restore the database of the given --chain (default: mainnet) from a snapshot file",

            ARG arg_restore_file: (Option<String>) = None,
            "[FILE]",
            "Path to the file to restore from",
        }

        CMD cmd_tools
        {
            "Tools",

            CMD cmd_tools_hash
            {
                "Hash a file using the Keccak-256 algorithm",

                ARG arg_tools_hash_file: (Option<String>) = None,
                "<FILE>",
                "File",
            }
        }

        CMD cmd_db
        {
            "Manage the database representing the state of the blockchain on this system",

            CMD cmd_db_kill {
                "Clean the database of the given --chain (default: mainnet)",
            }

            CMD cmd_db_reset {
                "Removes NUM latests blocks from the db",

                ARG arg_db_reset_num: (u32) = 10u32,
                "<NUM>",
                "Number of blocks to revert",
            }

        }
    }
    {
        // Global flags and arguments
        ["Operating Options"]
            ARG arg_mode: (String) = "last", or |c: &Config| c.parity.as_ref()?.mode.clone(),
            "--mode=[MODE]",
            "Set the operating mode. MODE can be one of: last - Uses the last-used mode, active if none; active - Parity continuously syncs the chain; passive - Parity syncs initially, then sleeps and wakes regularly to resync; dark - Parity syncs only when the JSON-RPC is active; offline - Parity doesn't sync.",

            ARG arg_mode_timeout: (u64) = 300u64, or |c: &Config| c.parity.as_ref()?.mode_timeout.clone(),
            "--mode-timeout=[SECS]",
            "Specify the number of seconds before inactivity timeout occurs when mode is dark or passive",

            ARG arg_mode_alarm: (u64) = 3600u64, or |c: &Config| c.parity.as_ref()?.mode_alarm.clone(),
            "--mode-alarm=[SECS]",
            "Specify the number of seconds before auto sleep reawake timeout occurs when mode is passive",

            ARG arg_chain: (String) = "foundation", or |c: &Config| c.parity.as_ref()?.chain.clone(),
            "--chain=[CHAIN]",
            "Specify the blockchain type. CHAIN may be either a JSON chain specification file or ethereum, poacore, xdai, volta, ewc, musicoin, ellaism, mix, callisto, morden, ropsten, kovan, rinkeby, goerli, poasokol, testnet, or dev.",

            ARG arg_keys_path: (String) = "$BASE/keys", or |c: &Config| c.parity.as_ref()?.keys_path.clone(),
            "--keys-path=[PATH]",
            "Specify the path for JSON key files to be found",

            ARG arg_identity: (String) = "", or |c: &Config| c.parity.as_ref()?.identity.clone(),
            "--identity=[NAME]",
            "Specify your node's name.",

            ARG arg_base_path: (Option<String>) = None, or |c: &Config| c.parity.as_ref()?.base_path.clone(),
            "-d, --base-path=[PATH]",
            "Specify the base data storage path.",

            ARG arg_db_path: (Option<String>) = None, or |c: &Config| c.parity.as_ref()?.db_path.clone(),
            "--db-path=[PATH]",
            "Specify the database directory path",

        ["Convenience Options"]
            FLAG flag_unsafe_expose: (bool) = false, or |c: &Config| c.misc.as_ref()?.unsafe_expose,
            "--unsafe-expose",
            "All servers will listen on external interfaces and will be remotely accessible. It's equivalent with setting the following: --[ws,jsonrpc,secretstore,stratum,secretstore-http]-interface=all --*-hosts=all    This option is UNSAFE and should be used with great care!",

            ARG arg_config: (String) = "$BASE/config.toml", or |_| None,
            "-c, --config=[CONFIG]",
            "Specify a configuration. CONFIG may be either a configuration file or a preset: dev, insecure, dev-insecure, mining, or non-standard-ports.",

            ARG arg_ports_shift: (u16) = 0u16, or |c: &Config| c.misc.as_ref()?.ports_shift,
            "--ports-shift=[SHIFT]",
            "Add SHIFT to all port numbers OpenEthereum is listening on. Includes network port and all servers (HTTP JSON-RPC, WebSockets JSON-RPC, SecretStore).",

        ["Account Options"]
            FLAG flag_fast_unlock: (bool) = false, or |c: &Config| c.account.as_ref()?.fast_unlock.clone(),
            "--fast-unlock",
            "Use drastically faster unlocking mode. This setting causes raw secrets to be stored unprotected in memory, so use with care.",

            ARG arg_keys_iterations: (u32) = 10240u32, or |c: &Config| c.account.as_ref()?.keys_iterations.clone(),
            "--keys-iterations=[NUM]",
            "Specify the number of iterations to use when deriving key from the password (bigger is more secure)",

            ARG arg_accounts_refresh: (u64) = 5u64, or |c: &Config| c.account.as_ref()?.refresh_time.clone(),
            "--accounts-refresh=[TIME]",
            "Specify the cache time of accounts read from disk. If you manage thousands of accounts set this to 0 to disable refresh.",

            ARG arg_unlock: (Option<String>) = None, or |c: &Config| c.account.as_ref()?.unlock.as_ref().map(|vec| vec.join(",")),
            "--unlock=[ACCOUNTS]",
            "Unlock ACCOUNTS for the duration of the execution. ACCOUNTS is a comma-delimited list of addresses.",

            ARG arg_password: (Vec<String>) = Vec::new(), or |c: &Config| c.account.as_ref()?.password.clone(),
            "--password=[FILE]...",
            "Provide a file containing a password for unlocking an account. Leading and trailing whitespace is trimmed.",

        ["UI Options"]
            ARG arg_ui_path: (String) = "$BASE/signer", or |c: &Config| c.ui.as_ref()?.path.clone(),
            "--ui-path=[PATH]",
            "Specify directory where Trusted UIs tokens should be stored.",

        ["Networking Options"]
            FLAG flag_no_warp: (bool) = false, or |c: &Config| c.network.as_ref()?.warp.clone().map(|w| !w),
            "--no-warp",
            "Disable syncing from the snapshot over the network.",

            FLAG flag_no_discovery: (bool) = false, or |c: &Config| c.network.as_ref()?.discovery.map(|d| !d).clone(),
            "--no-discovery",
            "Disable new peer discovery.",

            FLAG flag_reserved_only: (bool) = false, or |c: &Config| c.network.as_ref()?.reserved_only.clone(),
            "--reserved-only",
            "Connect only to reserved nodes.",

            FLAG flag_no_ancient_blocks: (bool) = false, or |_| None,
            "--no-ancient-blocks",
            "Disable downloading old blocks after snapshot restoration or warp sync. Not recommended.",

            ARG arg_warp_barrier: (Option<u64>) = None, or |c: &Config| c.network.as_ref()?.warp_barrier.clone(),
            "--warp-barrier=[NUM]",
            "When warp enabled never attempt regular sync before warping to block NUM.",

            ARG arg_port: (u16) = 30303u16, or |c: &Config| c.network.as_ref()?.port.clone(),
            "--port=[PORT]",
            "Override the port on which the node should listen.",

            ARG arg_interface: (String) = "all", or |c: &Config| c.network.as_ref()?.interface.clone(),
            "--interface=[IP]",
            "Network interfaces. Valid values are 'all', 'local' or the ip of the interface you want OpenEthereum to listen to.",

            ARG arg_min_peers: (Option<u16>) = None, or |c: &Config| c.network.as_ref()?.min_peers.clone(),
            "--min-peers=[NUM]",
            "Try to maintain at least NUM peers.",

            ARG arg_max_peers: (Option<u16>) = None, or |c: &Config| c.network.as_ref()?.max_peers.clone(),
            "--max-peers=[NUM]",
            "Allow up to NUM peers.",

            ARG arg_snapshot_peers: (u16) = 0u16, or |c: &Config| c.network.as_ref()?.snapshot_peers.clone(),
            "--snapshot-peers=[NUM]",
            "Allow additional NUM peers for a snapshot sync.",

            ARG arg_nat: (String) = "any", or |c: &Config| c.network.as_ref()?.nat.clone(),
            "--nat=[METHOD]",
            "Specify method to use for determining public address. Must be one of: any, none, upnp, extip:<IP>.",

            ARG arg_allow_ips: (String) = "all", or |c: &Config| c.network.as_ref()?.allow_ips.clone(),
            "--allow-ips=[FILTER]",
            "Filter outbound connections. Must be one of: private - connect to private network IP addresses only; public - connect to public network IP addresses only; all - connect to any IP address.",

            ARG arg_max_pending_peers: (u16) = 64u16, or |c: &Config| c.network.as_ref()?.max_pending_peers.clone(),
            "--max-pending-peers=[NUM]",
            "Allow up to NUM pending connections.",

            ARG arg_network_id: (Option<u64>) = None, or |c: &Config| c.network.as_ref()?.id.clone(),
            "--network-id=[INDEX]",
            "Override the network identifier from the chain we are on.",

            ARG arg_bootnodes: (Option<String>) = None, or |c: &Config| c.network.as_ref()?.bootnodes.as_ref().map(|vec| vec.join(",")),
            "--bootnodes=[NODES]",
            "Override the bootnodes from our chain. NODES should be comma-delimited enodes.",

            ARG arg_node_key: (Option<String>) = None, or |c: &Config| c.network.as_ref()?.node_key.clone(),
            "--node-key=[KEY]",
            "Specify node secret key, either as 64-character hex string or input to SHA3 operation.",

            ARG arg_reserved_peers: (Option<String>) = None, or |c: &Config| c.network.as_ref()?.reserved_peers.clone(),
            "--reserved-peers=[FILE]",
            "Provide a file containing enodes, one per line. These nodes will always have a reserved slot on top of the normal maximum peers.",

            CHECK |args: &Args| {
                if let (Some(max_peers), Some(min_peers)) = (args.arg_max_peers, args.arg_min_peers) {
                    if min_peers > max_peers {
                        return Err(ArgsError::PeerConfiguration);
                    }
                }

                Ok(())
            },

        ["API and Console Options – HTTP JSON-RPC"]
            FLAG flag_jsonrpc_allow_missing_blocks: (bool) = false, or |c: &Config| c.rpc.as_ref()?.allow_missing_blocks.clone(),
            "--jsonrpc-allow-missing-blocks",
            "RPC calls will return 'null' instead of an error if ancient block sync is still in progress and the block information requested could not be found",

            FLAG flag_no_jsonrpc: (bool) = false, or |c: &Config| c.rpc.as_ref()?.disable.clone(),
            "--no-jsonrpc",
            "Disable the HTTP JSON-RPC API server.",

            FLAG flag_jsonrpc_no_keep_alive: (bool) = false, or |c: &Config| c.rpc.as_ref()?.keep_alive,
            "--jsonrpc-no-keep-alive",
            "Disable HTTP/1.1 keep alive header. Disabling keep alive will prevent re-using the same TCP connection to fire multiple requests, recommended when using one request per connection.",

            FLAG flag_jsonrpc_experimental: (bool) = false, or |c: &Config| c.rpc.as_ref()?.experimental_rpcs.clone(),
            "--jsonrpc-experimental",
            "Enable experimental RPCs. Enable to have access to methods from unfinalised EIPs in all namespaces",

            ARG arg_jsonrpc_port: (u16) = 8545u16, or |c: &Config| c.rpc.as_ref()?.port.clone(),
            "--jsonrpc-port=[PORT]",
            "Specify the port portion of the HTTP JSON-RPC API server.",

            ARG arg_jsonrpc_interface: (String) = "local", or |c: &Config| c.rpc.as_ref()?.interface.clone(),
            "--jsonrpc-interface=[IP]",
            "Specify the hostname portion of the HTTP JSON-RPC API server, IP should be an interface's IP address, or all (all interfaces) or local.",

            ARG arg_jsonrpc_apis: (String) = "web3,eth,pubsub,net,parity,parity_pubsub,traces", or |c: &Config| c.rpc.as_ref()?.apis.as_ref().map(|vec| vec.join(",")),
            "--jsonrpc-apis=[APIS]",
            "Specify the APIs available through the HTTP JSON-RPC interface using a comma-delimited list of API names. Possible names are: all, safe, debug, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces",

            ARG arg_jsonrpc_hosts: (String) = "none", or |c: &Config| c.rpc.as_ref()?.hosts.as_ref().map(|vec| vec.join(",")),
            "--jsonrpc-hosts=[HOSTS]",
            "List of allowed Host header values. This option will validate the Host header sent by the browser, it is additional security against some attack vectors. Special options: \"all\", \"none\",.",

            ARG arg_jsonrpc_threads: (usize) = 4usize, or |c: &Config| c.rpc.as_ref()?.processing_threads,
            "--jsonrpc-threads=[THREADS]",
            "Turn on additional processing threads for JSON-RPC servers (all transports). Setting this to a non-zero value allows parallel execution of cpu-heavy queries.",

            ARG arg_jsonrpc_cors: (String) = "none", or |c: &Config| c.rpc.as_ref()?.cors.as_ref().map(|vec| vec.join(",")),
            "--jsonrpc-cors=[URL]",
            "Specify CORS header for HTTP JSON-RPC API responses. Special options: \"all\", \"none\".",

            ARG arg_jsonrpc_server_threads: (Option<usize>) = None, or |c: &Config| c.rpc.as_ref()?.server_threads,
            "--jsonrpc-server-threads=[NUM]",
            "Enables multiple threads handling incoming connections for HTTP JSON-RPC server.",

            ARG arg_jsonrpc_max_payload: (Option<usize>) = None, or |c: &Config| c.rpc.as_ref()?.max_payload,
            "--jsonrpc-max-payload=[MB]",
            "Specify maximum size for HTTP JSON-RPC requests in megabytes.",

            ARG arg_poll_lifetime: (u32) = 60u32, or |c: &Config| c.rpc.as_ref()?.poll_lifetime.clone(),
            "--poll-lifetime=[S]",
            "Set the RPC filter lifetime to S seconds. The filter has to be polled at least every S seconds , otherwise it is removed.",

        ["API and Console Options – WebSockets"]
            FLAG flag_no_ws: (bool) = false, or |c: &Config| c.websockets.as_ref()?.disable.clone(),
            "--no-ws",
            "Disable the WebSockets JSON-RPC server.",

            ARG arg_ws_port: (u16) = 8546u16, or |c: &Config| c.websockets.as_ref()?.port.clone(),
            "--ws-port=[PORT]",
            "Specify the port portion of the WebSockets JSON-RPC server.",

            ARG arg_ws_interface: (String) = "local", or |c: &Config| c.websockets.as_ref()?.interface.clone(),
            "--ws-interface=[IP]",
            "Specify the hostname portion of the WebSockets JSON-RPC server, IP should be an interface's IP address, or all (all interfaces) or local.",

            ARG arg_ws_apis: (String) = "web3,eth,pubsub,net,parity,parity_pubsub,traces", or |c: &Config| c.websockets.as_ref()?.apis.as_ref().map(|vec| vec.join(",")),
            "--ws-apis=[APIS]",
            "Specify the JSON-RPC APIs available through the WebSockets interface using a comma-delimited list of API names. Possible names are: all, safe, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces",

            ARG arg_ws_origins: (String) = "parity://*,chrome-extension://*,moz-extension://*", or |c: &Config| c.websockets.as_ref()?.origins.as_ref().map(|vec| vec.join(",")),
            "--ws-origins=[URL]",
            "Specify Origin header values allowed to connect. Special options: \"all\", \"none\".",

            ARG arg_ws_hosts: (String) = "none", or |c: &Config| c.websockets.as_ref()?.hosts.as_ref().map(|vec| vec.join(",")),
            "--ws-hosts=[HOSTS]",
            "List of allowed Host header values. This option will validate the Host header sent by the browser, it is additional security against some attack vectors. Special options: \"all\", \"none\".",

            ARG arg_ws_max_connections: (usize) = 100usize, or |c: &Config| c.websockets.as_ref()?.max_connections,
            "--ws-max-connections=[CONN]",
            "Maximum number of allowed concurrent WebSockets JSON-RPC connections.",

        ["Metrics"]
            FLAG flag_metrics: (bool) = false, or |c: &Config| c.metrics.as_ref()?.enable.clone(),
            "--metrics",
            "Enable prometheus metrics (only full client).",

            ARG arg_metrics_port: (u16) = 3000u16, or |c: &Config| c.metrics.as_ref()?.port.clone(),
            "--metrics-port=[PORT]",
            "Specify the port portion of the metrics server.",

            ARG arg_metrics_interface: (String) = "local", or |c: &Config| c.metrics.as_ref()?.interface.clone(),
            "--metrics-interface=[IP]",
            "Specify the hostname portion of the metrics server, IP should be an interface's IP address, or all (all interfaces) or local.",

        ["API and Console Options – IPC"]
            FLAG flag_no_ipc: (bool) = false, or |c: &Config| c.ipc.as_ref()?.disable.clone(),
            "--no-ipc",
            "Disable JSON-RPC over IPC service.",

            ARG arg_ipc_path: (String) = if cfg!(windows) { r"\\.\pipe\jsonrpc.ipc" } else { "$BASE/jsonrpc.ipc" }, or |c: &Config| c.ipc.as_ref()?.path.clone(),
            "--ipc-path=[PATH]",
            "Specify custom path for JSON-RPC over IPC service.",

            ARG arg_ipc_apis: (String) = "web3,eth,pubsub,net,parity,parity_pubsub,parity_accounts,traces", or |c: &Config| c.ipc.as_ref()?.apis.as_ref().map(|vec| vec.join(",")),
            "--ipc-apis=[APIS]",
            "Specify custom API set available via JSON-RPC over IPC using a comma-delimited list of API names. Possible names are: all, safe, web3, net, eth, pubsub, personal, signer, parity, parity_pubsub, parity_accounts, parity_set, traces, secretstore. You can also disable a specific API by putting '-' in the front, example: all,-personal. 'safe' enables the following APIs: web3, net, eth, pubsub, parity, parity_pubsub, traces",

        ["Secret Store Options"]
            FLAG flag_no_secretstore: (bool) = false, or |c: &Config| c.secretstore.as_ref()?.disable.clone(),
            "--no-secretstore",
            "Disable Secret Store functionality.",

            FLAG flag_no_secretstore_http: (bool) = false, or |c: &Config| c.secretstore.as_ref()?.disable_http.clone(),
            "--no-secretstore-http",
            "Disable Secret Store HTTP API.",

            FLAG flag_no_secretstore_auto_migrate: (bool) = false, or |c: &Config| c.secretstore.as_ref()?.disable_auto_migrate.clone(),
            "--no-secretstore-auto-migrate",
            "Do not run servers set change session automatically when servers set changes. This option has no effect when servers set is read from configuration file.",

            ARG arg_secretstore_acl_contract: (Option<String>) = Some("registry".into()), or |c: &Config| c.secretstore.as_ref()?.acl_contract.clone(),
            "--secretstore-acl-contract=[SOURCE]",
            "Secret Store permissioning contract address source: none, registry (contract address is read from 'secretstore_acl_checker' entry in registry) or address.",

            ARG arg_secretstore_contract: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.service_contract.clone(),
            "--secretstore-contract=[SOURCE]",
            "Secret Store Service contract address source: none, registry (contract address is read from 'secretstore_service' entry in registry) or address.",

            ARG arg_secretstore_srv_gen_contract: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.service_contract_srv_gen.clone(),
            "--secretstore-srv-gen-contract=[SOURCE]",
            "Secret Store Service server key generation contract address source: none, registry (contract address is read from 'secretstore_service_srv_gen' entry in registry) or address.",

            ARG arg_secretstore_srv_retr_contract: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.service_contract_srv_retr.clone(),
            "--secretstore-srv-retr-contract=[SOURCE]",
            "Secret Store Service server key retrieval contract address source: none, registry (contract address is read from 'secretstore_service_srv_retr' entry in registry) or address.",

            ARG arg_secretstore_doc_store_contract: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.service_contract_doc_store.clone(),
            "--secretstore-doc-store-contract=[SOURCE]",
            "Secret Store Service document key store contract address source: none, registry (contract address is read from 'secretstore_service_doc_store' entry in registry) or address.",

            ARG arg_secretstore_doc_sretr_contract: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.service_contract_doc_sretr.clone(),
            "--secretstore-doc-sretr-contract=[SOURCE]",
            "Secret Store Service document key shadow retrieval contract address source: none, registry (contract address is read from 'secretstore_service_doc_sretr' entry in registry) or address.",

            ARG arg_secretstore_nodes: (String) = "", or |c: &Config| c.secretstore.as_ref()?.nodes.as_ref().map(|vec| vec.join(",")),
            "--secretstore-nodes=[NODES]",
            "Comma-separated list of other secret store cluster nodes in form NODE_PUBLIC_KEY_IN_HEX@NODE_IP_ADDR:NODE_PORT.",

            ARG arg_secretstore_server_set_contract: (Option<String>) = Some("registry".into()), or |c: &Config| c.secretstore.as_ref()?.server_set_contract.clone(),
            "--secretstore-server-set-contract=[SOURCE]",
            "Secret Store server set contract address source: none, registry (contract address is read from 'secretstore_server_set' entry in registry) or address.",

            ARG arg_secretstore_interface: (String) = "local", or |c: &Config| c.secretstore.as_ref()?.interface.clone(),
            "--secretstore-interface=[IP]",
            "Specify the hostname portion for listening to Secret Store Key Server internal requests, IP should be an interface's IP address, or local.",

            ARG arg_secretstore_port: (u16) = 8083u16, or |c: &Config| c.secretstore.as_ref()?.port.clone(),
            "--secretstore-port=[PORT]",
            "Specify the port portion for listening to Secret Store Key Server internal requests.",

            ARG arg_secretstore_http_interface: (String) = "local", or |c: &Config| c.secretstore.as_ref()?.http_interface.clone(),
            "--secretstore-http-interface=[IP]",
            "Specify the hostname portion for listening to Secret Store Key Server HTTP requests, IP should be an interface's IP address, or local.",

            ARG arg_secretstore_http_port: (u16) = 8082u16, or |c: &Config| c.secretstore.as_ref()?.http_port.clone(),
            "--secretstore-http-port=[PORT]",
            "Specify the port portion for listening to Secret Store Key Server HTTP requests.",

            ARG arg_secretstore_path: (String) = "$BASE/secretstore", or |c: &Config| c.secretstore.as_ref()?.path.clone(),
            "--secretstore-path=[PATH]",
            "Specify directory where Secret Store should save its data.",

            ARG arg_secretstore_secret: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.self_secret.clone(),
            "--secretstore-secret=[SECRET]",
            "Hex-encoded secret key of this node.",

            ARG arg_secretstore_admin_public: (Option<String>) = None, or |c: &Config| c.secretstore.as_ref()?.admin_public.clone(),
            "--secretstore-admin=[PUBLIC]",
            "Hex-encoded public key of secret store administrator.",

        ["Sealing/Mining Options"]
            FLAG flag_force_sealing: (bool) = false, or |c: &Config| c.mining.as_ref()?.force_sealing.clone(),
            "--force-sealing",
            "Force the node to author new blocks as if it were always sealing/mining.",

            FLAG flag_reseal_on_uncle: (bool) = false, or |c: &Config| c.mining.as_ref()?.reseal_on_uncle.clone(),
            "--reseal-on-uncle",
            "Force the node to author new blocks when a new uncle block is imported.",

            FLAG flag_remove_solved: (bool) = false, or |c: &Config| c.mining.as_ref()?.remove_solved.clone(),
            "--remove-solved",
            "Move solved blocks from the work package queue instead of cloning them. This gives a slightly faster import speed, but means that extra solutions submitted for the same work package will go unused.",

            FLAG flag_tx_queue_no_unfamiliar_locals: (bool) = false, or |c: &Config| c.mining.as_ref()?.tx_queue_no_unfamiliar_locals.clone(),
            "--tx-queue-no-unfamiliar-locals",
            "Local transactions sent through JSON-RPC (HTTP, WebSockets, etc) will be treated as 'external' if the sending account is unknown.",

            FLAG flag_tx_queue_no_early_reject: (bool) = false, or |c: &Config| c.mining.as_ref()?.tx_queue_no_early_reject.clone(),
            "--tx-queue-no-early-reject",
            "Disables transaction queue optimization to early reject transactions below minimal effective gas price. This allows local transactions to always enter the pool, despite it being full, but requires additional ecrecover on every transaction.",

            FLAG flag_refuse_service_transactions: (bool) = false, or |c: &Config| c.mining.as_ref()?.refuse_service_transactions.clone(),
            "--refuse-service-transactions",
            "Always refuse service transactions.",

            FLAG flag_infinite_pending_block: (bool) = false, or |c: &Config| c.mining.as_ref()?.infinite_pending_block.clone(),
            "--infinite-pending-block",
            "Pending block will be created with maximal possible gas limit and will execute all transactions in the queue. Note that such block is invalid and should never be attempted to be mined.",

            FLAG flag_no_persistent_txqueue: (bool) = false, or |c: &Config| c.parity.as_ref()?.no_persistent_txqueue,
            "--no-persistent-txqueue",
            "Don't save pending local transactions to disk to be restored whenever the node restarts.",

            FLAG flag_stratum: (bool) = false, or |c: &Config| Some(c.stratum.is_some()),
            "--stratum",
            "Run Stratum server for miner push notification.",

            ARG arg_reseal_on_txs: (String) = "own", or |c: &Config| c.mining.as_ref()?.reseal_on_txs.clone(),
            "--reseal-on-txs=[SET]",
            "Specify which transactions should force the node to reseal a block. SET is one of: none - never reseal on new transactions; own - reseal only on a new local transaction; ext - reseal only on a new external transaction; all - reseal on all new transactions.",

            ARG arg_reseal_min_period: (u64) = 2000u64, or |c: &Config| c.mining.as_ref()?.reseal_min_period.clone(),
            "--reseal-min-period=[MS]",
            "Specify the minimum time between reseals from incoming transactions. MS is time measured in milliseconds.",

            ARG arg_reseal_max_period: (u64) = 120000u64, or |c: &Config| c.mining.as_ref()?.reseal_max_period.clone(),
            "--reseal-max-period=[MS]",
            "Specify the maximum time since last block to enable force-sealing. MS is time measured in milliseconds.",

            ARG arg_work_queue_size: (usize) = 20usize, or |c: &Config| c.mining.as_ref()?.work_queue_size.clone(),
            "--work-queue-size=[ITEMS]",
            "Specify the number of historical work packages which are kept cached lest a solution is found for them later. High values take more memory but result in fewer unusable solutions.",

            ARG arg_relay_set: (String) = "cheap", or |c: &Config| c.mining.as_ref()?.relay_set.clone(),
            "--relay-set=[SET]",
            "Set of transactions to relay. SET may be: cheap - Relay any transaction in the queue (this may include invalid transactions); strict - Relay only executed transactions (this guarantees we don't relay invalid transactions, but means we relay nothing if not mining); lenient - Same as strict when mining, and cheap when not.",

            ARG arg_usd_per_tx: (String) = "0.0001", or |c: &Config| c.mining.as_ref()?.usd_per_tx.clone(),
            "--usd-per-tx=[USD]",
            "Amount of USD to be paid for a basic transaction. The minimum gas price is set accordingly.",

            ARG arg_usd_per_eth: (String) = "auto", or |c: &Config| c.mining.as_ref()?.usd_per_eth.clone(),
            "--usd-per-eth=[SOURCE]",
            "USD value of a single ETH. SOURCE may be either an amount in USD, a web service or 'auto' to use each web service in turn and fallback on the last known good value.",

            ARG arg_price_update_period: (String) = "hourly", or |c: &Config| c.mining.as_ref()?.price_update_period.clone(),
            "--price-update-period=[T]",
            "T will be allowed to pass between each gas price update. T may be daily, hourly, a number of seconds, or a time string of the form \"2 days\", \"30 minutes\" etc..",

            ARG arg_gas_floor_target: (String) = "8000000", or |c: &Config| c.mining.as_ref()?.gas_floor_target.clone(),
            "--gas-floor-target=[GAS]",
            "Amount of gas per block to target when sealing a new block.",

            ARG arg_gas_cap: (String) = "10000000", or |c: &Config| c.mining.as_ref()?.gas_cap.clone(),
            "--gas-cap=[GAS]",
            "A cap on how large we will raise the gas limit per block due to transaction volume.",

            ARG arg_tx_queue_mem_limit: (u32) = 4u32, or |c: &Config| c.mining.as_ref()?.tx_queue_mem_limit.clone(),
            "--tx-queue-mem-limit=[MB]",
            "Maximum amount of memory that can be used by the transaction queue. Setting this parameter to 0 disables limiting.",

            ARG arg_tx_queue_size: (usize) = 8_192usize, or |c: &Config| c.mining.as_ref()?.tx_queue_size.clone(),
            "--tx-queue-size=[LIMIT]",
            "Maximum amount of transactions in the queue (waiting to be included in next block).",

            ARG arg_tx_queue_per_sender: (Option<usize>) = None, or |c: &Config| c.mining.as_ref()?.tx_queue_per_sender.clone(),
            "--tx-queue-per-sender=[LIMIT]",
            "Maximum number of transactions per sender in the queue. By default it's 1% of the entire queue, but not less than 16.",

            ARG arg_tx_queue_locals: (Option<String>) = None, or |c: &Config| helpers::join_set(c.mining.as_ref()?.tx_queue_locals.as_ref()),
            "--tx-queue-locals=[ACCOUNTS]",
            "Specify local accounts for which transactions are prioritized in the queue. ACCOUNTS is a comma-delimited list of addresses.",

            ARG arg_tx_queue_strategy: (String) = "gas_price", or |c: &Config| c.mining.as_ref()?.tx_queue_strategy.clone(),
            "--tx-queue-strategy=[S]",
            "Prioritization strategy used to order transactions in the queue. S may be: gas_price - Prioritize txs with high gas price",

            ARG arg_stratum_interface: (String) = "local", or |c: &Config| c.stratum.as_ref()?.interface.clone(),
            "--stratum-interface=[IP]",
            "Interface address for Stratum server.",

            ARG arg_stratum_port: (u16) = 8008u16, or |c: &Config| c.stratum.as_ref()?.port.clone(),
            "--stratum-port=[PORT]",
            "Port for Stratum server to listen on.",

            ARG arg_min_gas_price: (Option<u64>) = None, or |c: &Config| c.mining.as_ref()?.min_gas_price.clone(),
            "--min-gas-price=[STRING]",
            "Minimum amount of Wei per GAS to be paid for a transaction to be accepted for mining. Overrides --usd-per-tx.",

            ARG arg_gas_price_percentile: (usize) = 50usize, or |c: &Config| c.mining.as_ref()?.gas_price_percentile,
            "--gas-price-percentile=[PCT]",
            "Set PCT percentile gas price value from last 100 blocks as default gas price when sending transactions.",

            ARG arg_author: (Option<String>) = None, or |c: &Config| c.mining.as_ref()?.author.clone(),
            "--author=[ADDRESS]",
            "Specify the block author (aka \"coinbase\") address for sending block rewards from sealed blocks. NOTE: MINING WILL NOT WORK WITHOUT THIS OPTION.", // Sealing/Mining Option

            ARG arg_engine_signer: (Option<String>) = None, or |c: &Config| c.mining.as_ref()?.engine_signer.clone(),
            "--engine-signer=[ADDRESS]",
            "Specify the address which should be used to sign consensus messages and issue blocks. Relevant only to non-PoW chains.",

            ARG arg_tx_gas_limit: (Option<String>) = None, or |c: &Config| c.mining.as_ref()?.tx_gas_limit.clone(),
            "--tx-gas-limit=[GAS]",
            "Apply a limit of GAS as the maximum amount of gas a single transaction may have for it to be mined.",

            ARG arg_tx_time_limit: (Option<u64>) = None, or |c: &Config| c.mining.as_ref()?.tx_time_limit.clone(),
            "--tx-time-limit=[MS]",
            "Maximal time for processing single transaction. If enabled senders of transactions offending the limit will get other transactions penalized.",

            ARG arg_extra_data: (Option<String>) = None, or |c: &Config| c.mining.as_ref()?.extra_data.clone(),
            "--extra-data=[STRING]",
            "Specify a custom extra-data for authored blocks, no more than 32 characters.",

            ARG arg_notify_work: (Option<String>) = None, or |c: &Config| c.mining.as_ref()?.notify_work.as_ref().map(|vec| vec.join(",")),
            "--notify-work=[URLS]",
            "URLs to which work package notifications are pushed. URLS should be a comma-delimited list of HTTP URLs.",

            ARG arg_stratum_secret: (Option<String>) = None, or |c: &Config| c.stratum.as_ref()?.secret.clone(),
            "--stratum-secret=[STRING]",
            "Secret for authorizing Stratum server for peers.",

            ARG arg_max_round_blocks_to_import: (usize) = 12usize, or |c: &Config| c.mining.as_ref()?.max_round_blocks_to_import.clone(),
            "--max-round-blocks-to-import=[S]",
            "Maximal number of blocks to import for each import round.",

        ["Internal Options"]
            FLAG flag_can_restart: (bool) = false, or |_| None,
            "--can-restart",
            "Executable will auto-restart if exiting with 69",

        ["Miscellaneous Options"]
            FLAG flag_no_color: (bool) = false, or |c: &Config| c.misc.as_ref()?.color.map(|c| !c).clone(),
            "--no-color",
            "Don't use terminal color codes in output.",

            FLAG flag_version: (bool) = false, or |_| None,
            "-v, --version",
            "Show information about version.",

            FLAG flag_no_config: (bool) = false, or |_| None,
            "--no-config",
            "Don't load a configuration file.",

            ARG arg_logging: (Option<String>) = None, or |c: &Config| c.misc.as_ref()?.logging.clone(),
            "-l, --logging=[LOGGING]",
            "Specify the general logging level (error, warn, info, debug or trace). It can also be set for a specific module, example: '-l sync=debug,rpc=trace'",

            ARG arg_log_file: (Option<String>) = None, or |c: &Config| c.misc.as_ref()?.log_file.clone(),
            "--log-file=[FILENAME]",
            "Specify a filename into which logging should be appended.",

        ["Footprint Options"]
            FLAG flag_scale_verifiers: (bool) = false, or |c: &Config| c.footprint.as_ref()?.scale_verifiers.clone(),
            "--scale-verifiers",
            "Automatically scale amount of verifier threads based on workload. Not guaranteed to be faster.",

            ARG arg_tracing: (String) = "auto", or |c: &Config| c.footprint.as_ref()?.tracing.clone(),
            "--tracing=[BOOL]",
            "Indicates if full transaction tracing should be enabled. Works only if client had been fully synced with tracing enabled. BOOL may be one of auto, on, off. auto uses last used value of this option (off if it does not exist).", // footprint option

            ARG arg_pruning: (String) = "auto", or |c: &Config| c.footprint.as_ref()?.pruning.clone(),
            "--pruning=[METHOD]",
            "Configure pruning of the state/storage trie. METHOD may be one of auto, archive, fast: archive - keep all state trie data. No pruning. fast - maintain journal overlay. Fast but 50MB used. auto - use the method most recently synced or default to fast if none synced.",

            ARG arg_pruning_history: (u64) = 64u64, or |c: &Config| c.footprint.as_ref()?.pruning_history.clone(),
            "--pruning-history=[NUM]",
            "Set a minimum number of recent states to keep in memory when pruning is active.",

            ARG arg_pruning_memory: (usize) = 32usize, or |c: &Config| c.footprint.as_ref()?.pruning_memory.clone(),
            "--pruning-memory=[MB]",
            "The ideal amount of memory in megabytes to use to store recent states. As many states as possible will be kept within this limit, and at least --pruning-history states will always be kept.",

            ARG arg_cache_size_db: (u32) = 128u32, or |c: &Config| c.footprint.as_ref()?.cache_size_db.clone(),
            "--cache-size-db=[MB]",
            "Override database cache size.",

            ARG arg_cache_size_blocks: (u32) = 8u32, or |c: &Config| c.footprint.as_ref()?.cache_size_blocks.clone(),
            "--cache-size-blocks=[MB]",
            "Specify the preferred size of the blockchain cache in megabytes.",

            ARG arg_cache_size_queue: (u32) = 40u32, or |c: &Config| c.footprint.as_ref()?.cache_size_queue.clone(),
            "--cache-size-queue=[MB]",
            "Specify the maximum size of memory to use for block queue.",

            ARG arg_cache_size_state: (u32) = 25u32, or |c: &Config| c.footprint.as_ref()?.cache_size_state.clone(),
            "--cache-size-state=[MB]",
            "Specify the maximum size of memory to use for the state cache.",

            ARG arg_db_compaction: (String) = "auto", or |c: &Config| c.footprint.as_ref()?.db_compaction.clone(),
            "--db-compaction=[TYPE]",
            "Database compaction type. TYPE may be one of: ssd - suitable for SSDs and fast HDDs; hdd - suitable for slow HDDs; auto - determine automatically.",

            ARG arg_fat_db: (String) = "auto", or |c: &Config| c.footprint.as_ref()?.fat_db.clone(),
            "--fat-db=[BOOL]",
            "Build appropriate information to allow enumeration of all accounts and storage keys. Doubles the size of the state database. BOOL may be one of on, off or auto.",

            ARG arg_cache_size: (Option<u32>) = None, or |c: &Config| c.footprint.as_ref()?.cache_size.clone(),
            "--cache-size=[MB]",
            "Set total amount of discretionary memory to use for the entire system, overrides other cache and queue options.",

            ARG arg_num_verifiers: (Option<usize>) = None, or |c: &Config| c.footprint.as_ref()?.num_verifiers.clone(),
            "--num-verifiers=[INT]",
            "Amount of verifier threads to use or to begin with, if verifier auto-scaling is enabled.",

        ["Import/export Options"]
            FLAG flag_no_seal_check: (bool) = false, or |_| None,
            "--no-seal-check",
            "Skip block seal check.",

        ["Snapshot Options"]
            FLAG flag_enable_snapshotting: (bool) = false, or |c: &Config| c.snapshots.as_ref()?.enable.clone(),
            "--enable-snapshotting",
            "Enable automated snapshots which usually occur once every 5000 blocks.",

            ARG arg_snapshot_threads: (Option<usize>) = None, or |c: &Config| c.snapshots.as_ref()?.processing_threads,
            "--snapshot-threads=[NUM]",
            "Enables multiple threads for snapshots creation.",
    }
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    parity: Option<Operating>,
    account: Option<Account>,
    ui: Option<Ui>,
    network: Option<Network>,
    rpc: Option<Rpc>,
    websockets: Option<Ws>,
    ipc: Option<Ipc>,
    secretstore: Option<SecretStore>,
    mining: Option<Mining>,
    footprint: Option<Footprint>,
    snapshots: Option<Snapshots>,
    misc: Option<Misc>,
    stratum: Option<Stratum>,
    metrics: Option<Metrics>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operating {
    mode: Option<String>,
    mode_timeout: Option<u64>,
    mode_alarm: Option<u64>,
    chain: Option<String>,
    base_path: Option<String>,
    db_path: Option<String>,
    keys_path: Option<String>,
    identity: Option<String>,
    no_persistent_txqueue: Option<bool>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Account {
    unlock: Option<Vec<String>>,
    password: Option<Vec<String>>,
    keys_iterations: Option<u32>,
    refresh_time: Option<u64>,
    fast_unlock: Option<bool>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ui {
    path: Option<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Network {
    warp: Option<bool>,
    warp_barrier: Option<u64>,
    port: Option<u16>,
    interface: Option<String>,
    min_peers: Option<u16>,
    max_peers: Option<u16>,
    snapshot_peers: Option<u16>,
    max_pending_peers: Option<u16>,
    nat: Option<String>,
    allow_ips: Option<String>,
    id: Option<u64>,
    bootnodes: Option<Vec<String>>,
    discovery: Option<bool>,
    node_key: Option<String>,
    reserved_peers: Option<String>,
    reserved_only: Option<bool>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rpc {
    disable: Option<bool>,
    port: Option<u16>,
    interface: Option<String>,
    cors: Option<Vec<String>>,
    apis: Option<Vec<String>>,
    hosts: Option<Vec<String>>,
    server_threads: Option<usize>,
    processing_threads: Option<usize>,
    max_payload: Option<usize>,
    keep_alive: Option<bool>,
    experimental_rpcs: Option<bool>,
    poll_lifetime: Option<u32>,
    allow_missing_blocks: Option<bool>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ws {
    disable: Option<bool>,
    port: Option<u16>,
    interface: Option<String>,
    apis: Option<Vec<String>>,
    origins: Option<Vec<String>>,
    hosts: Option<Vec<String>>,
    max_connections: Option<usize>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ipc {
    disable: Option<bool>,
    path: Option<String>,
    apis: Option<Vec<String>>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metrics {
    enable: Option<bool>,
    port: Option<u16>,
    interface: Option<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SecretStore {
    disable: Option<bool>,
    disable_http: Option<bool>,
    disable_auto_migrate: Option<bool>,
    acl_contract: Option<String>,
    service_contract: Option<String>,
    service_contract_srv_gen: Option<String>,
    service_contract_srv_retr: Option<String>,
    service_contract_doc_store: Option<String>,
    service_contract_doc_sretr: Option<String>,
    self_secret: Option<String>,
    admin_public: Option<String>,
    nodes: Option<Vec<String>>,
    server_set_contract: Option<String>,
    interface: Option<String>,
    port: Option<u16>,
    http_interface: Option<String>,
    http_port: Option<u16>,
    path: Option<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mining {
    author: Option<String>,
    engine_signer: Option<String>,
    force_sealing: Option<bool>,
    reseal_on_uncle: Option<bool>,
    reseal_on_txs: Option<String>,
    reseal_min_period: Option<u64>,
    reseal_max_period: Option<u64>,
    work_queue_size: Option<usize>,
    tx_gas_limit: Option<String>,
    tx_time_limit: Option<u64>,
    relay_set: Option<String>,
    min_gas_price: Option<u64>,
    gas_price_percentile: Option<usize>,
    usd_per_tx: Option<String>,
    usd_per_eth: Option<String>,
    price_update_period: Option<String>,
    gas_floor_target: Option<String>,
    gas_cap: Option<String>,
    extra_data: Option<String>,
    tx_queue_size: Option<usize>,
    tx_queue_per_sender: Option<usize>,
    tx_queue_mem_limit: Option<u32>,
    tx_queue_locals: Option<HashSet<String>>,
    tx_queue_strategy: Option<String>,
    tx_queue_ban_count: Option<u16>,
    tx_queue_ban_time: Option<u16>,
    tx_queue_no_unfamiliar_locals: Option<bool>,
    tx_queue_no_early_reject: Option<bool>,
    remove_solved: Option<bool>,
    notify_work: Option<Vec<String>>,
    refuse_service_transactions: Option<bool>,
    infinite_pending_block: Option<bool>,
    max_round_blocks_to_import: Option<usize>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stratum {
    interface: Option<String>,
    port: Option<u16>,
    secret: Option<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Footprint {
    tracing: Option<String>,
    pruning: Option<String>,
    pruning_history: Option<u64>,
    pruning_memory: Option<usize>,
    fast_and_loose: Option<bool>,
    cache_size: Option<u32>,
    cache_size_db: Option<u32>,
    cache_size_blocks: Option<u32>,
    cache_size_queue: Option<u32>,
    cache_size_state: Option<u32>,
    db_compaction: Option<String>,
    fat_db: Option<String>,
    scale_verifiers: Option<bool>,
    num_verifiers: Option<usize>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshots {
    enable: Option<bool>,
    processing_threads: Option<usize>,
}

#[derive(Default, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Misc {
    logging: Option<String>,
    log_file: Option<String>,
    color: Option<bool>,
    ports_shift: Option<u16>,
    unsafe_expose: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{
        Account, Args, ArgsError, Config, Footprint, Ipc, Metrics, Mining, Misc, Network,
        Operating, Rpc, SecretStore, Snapshots, Ws,
    };
    use clap::ErrorKind as ClapErrorKind;
    use toml;

    #[test]
    fn should_accept_any_argument_order() {
        let args = Args::parse(&["openethereum", "--no-warp", "account", "list"]).unwrap();
        assert_eq!(args.flag_no_warp, true);

        let args = Args::parse(&["openethereum", "account", "list", "--no-warp"]).unwrap();
        assert_eq!(args.flag_no_warp, true);

        let args = Args::parse(&["openethereum", "--chain=dev", "account", "list"]).unwrap();
        assert_eq!(args.arg_chain, "dev");

        let args = Args::parse(&["openethereum", "account", "list", "--chain=dev"]).unwrap();
        assert_eq!(args.arg_chain, "dev");
    }

    #[test]
    fn should_reject_invalid_values() {
        let args = Args::parse(&["openethereum", "--jsonrpc-port=8545"]);
        assert!(args.is_ok());

        let args = Args::parse(&["openethereum", "--jsonrpc-port=asd"]);
        assert!(args.is_err());
    }

    #[test]
    fn should_parse_args_and_flags() {
        let args = Args::parse(&["openethereum", "--no-warp"]).unwrap();
        assert_eq!(args.flag_no_warp, true);

        let args = Args::parse(&["openethereum", "--pruning", "archive"]).unwrap();
        assert_eq!(args.arg_pruning, "archive");

        let args = Args::parse(&["openethereum", "export", "state", "--no-storage"]).unwrap();
        assert_eq!(args.flag_export_state_no_storage, true);

        let args =
            Args::parse(&["openethereum", "export", "state", "--min-balance", "123"]).unwrap();
        assert_eq!(args.arg_export_state_min_balance, Some("123".to_string()));
    }

    #[test]
    fn should_exit_gracefully_on_unknown_argument() {
        let result = Args::parse(&["openethereum", "--please-exit-gracefully"]);
        assert!(match result {
            Err(ArgsError::Clap(ref clap_error))
                if clap_error.kind == ClapErrorKind::UnknownArgument =>
                true,
            _ => false,
        });
    }

    #[test]
    fn should_use_subcommand_arg_default() {
        let args = Args::parse(&["openethereum", "export", "state", "--at", "123"]).unwrap();
        assert_eq!(args.arg_export_state_at, "123");
        assert_eq!(args.arg_snapshot_at, "latest");

        let args = Args::parse(&["openethereum", "snapshot", "--at", "123", "file.dump"]).unwrap();
        assert_eq!(args.arg_snapshot_at, "123");
        assert_eq!(args.arg_export_state_at, "latest");

        let args = Args::parse(&["openethereum", "export", "state"]).unwrap();
        assert_eq!(args.arg_snapshot_at, "latest");
        assert_eq!(args.arg_export_state_at, "latest");

        let args = Args::parse(&["openethereum", "snapshot", "file.dump"]).unwrap();
        assert_eq!(args.arg_snapshot_at, "latest");
        assert_eq!(args.arg_export_state_at, "latest");
    }

    #[test]
    fn should_parse_multiple_values() {
        let args = Args::parse(&["openethereum", "account", "import", "~/1", "~/2"]).unwrap();
        assert_eq!(
            args.arg_account_import_path,
            Some(vec!["~/1".to_owned(), "~/2".to_owned()])
        );

        let args = Args::parse(&["openethereum", "account", "import", "~/1,ext"]).unwrap();
        assert_eq!(
            args.arg_account_import_path,
            Some(vec!["~/1,ext".to_owned()])
        );

        let args = Args::parse(&[
            "openethereum",
            "--secretstore-nodes",
            "abc@127.0.0.1:3333,cde@10.10.10.10:4444",
        ])
        .unwrap();
        assert_eq!(
            args.arg_secretstore_nodes,
            "abc@127.0.0.1:3333,cde@10.10.10.10:4444"
        );

        let args = Args::parse(&[
            "openethereum",
            "--password",
            "~/.safe/1",
            "--password",
            "~/.safe/2",
        ])
        .unwrap();
        assert_eq!(
            args.arg_password,
            vec!["~/.safe/1".to_owned(), "~/.safe/2".to_owned()]
        );

        let args = Args::parse(&["openethereum", "--password", "~/.safe/1,~/.safe/2"]).unwrap();
        assert_eq!(
            args.arg_password,
            vec!["~/.safe/1".to_owned(), "~/.safe/2".to_owned()]
        );
    }

    #[test]
    fn should_parse_global_args_with_subcommand() {
        let args = Args::parse(&["openethereum", "--chain", "dev", "account", "list"]).unwrap();
        assert_eq!(args.arg_chain, "dev".to_owned());
    }

    #[test]
    fn should_parse_args_and_include_config() {
        // given
        let mut config = Config::default();
        let mut operating = Operating::default();
        operating.chain = Some("goerli".into());
        config.parity = Some(operating);

        // when
        let args = Args::parse_with_config(&["openethereum"], config).unwrap();

        // then
        assert_eq!(args.arg_chain, "goerli".to_owned());
    }

    #[test]
    fn should_not_use_config_if_cli_is_provided() {
        // given
        let mut config = Config::default();
        let mut operating = Operating::default();
        operating.chain = Some("goerli".into());
        config.parity = Some(operating);

        // when
        let args = Args::parse_with_config(&["openethereum", "--chain", "xyz"], config).unwrap();

        // then
        assert_eq!(args.arg_chain, "xyz".to_owned());
    }

    #[test]
    fn should_use_config_if_cli_is_missing() {
        let mut config = Config::default();
        let mut footprint = Footprint::default();
        footprint.pruning_history = Some(128);
        config.footprint = Some(footprint);

        // when
        let args = Args::parse_with_config(&["openethereum"], config).unwrap();

        // then
        assert_eq!(args.arg_pruning_history, 128);
    }

    #[test]
    fn should_parse_full_config() {
        // given
        let config = toml::from_str(include_str!("./tests/config.full.toml")).unwrap();

        // when
        let args = Args::parse_with_config(&["openethereum", "--chain", "xyz"], config).unwrap();

        // then
        assert_eq!(
            args,
            Args {
                // Commands
                cmd_daemon: false,
                cmd_account: false,
                cmd_account_new: false,
                cmd_account_list: false,
                cmd_account_import: false,
                cmd_wallet: false,
                cmd_wallet_import: false,
                cmd_import: false,
                cmd_export: false,
                cmd_export_blocks: false,
                cmd_export_state: false,
                cmd_signer: false,
                cmd_signer_list: false,
                cmd_signer_sign: false,
                cmd_signer_reject: false,
                cmd_signer_new_token: false,
                cmd_snapshot: false,
                cmd_restore: false,
                cmd_tools: false,
                cmd_tools_hash: false,
                cmd_db: false,
                cmd_db_kill: false,
                cmd_db_reset: false,

                // Arguments
                arg_daemon_pid_file: None,
                arg_import_file: None,
                arg_import_format: None,
                arg_export_blocks_file: None,
                arg_export_blocks_format: None,
                arg_export_state_file: None,
                arg_export_state_format: None,
                arg_snapshot_file: None,
                arg_restore_file: None,
                arg_tools_hash_file: None,

                arg_signer_sign_id: None,
                arg_signer_reject_id: None,
                arg_account_import_path: None,
                arg_wallet_import_path: None,
                arg_db_reset_num: 10,

                // -- Operating Options
                arg_mode: "last".into(),
                arg_mode_timeout: 300u64,
                arg_mode_alarm: 3600u64,
                arg_chain: "xyz".into(),
                arg_base_path: Some("$HOME/.parity".into()),
                arg_db_path: Some("$HOME/.parity/chains".into()),
                arg_keys_path: "$HOME/.parity/keys".into(),
                arg_identity: "".into(),
                flag_no_persistent_txqueue: false,

                // -- Convenience Options
                arg_config: "$BASE/config.toml".into(),
                arg_ports_shift: 0,
                flag_unsafe_expose: false,

                // -- Account Options
                arg_unlock: Some("0xdeadbeefcafe0000000000000000000000000000".into()),
                arg_password: vec!["~/.safe/password.file".into()],
                arg_keys_iterations: 10240u32,
                arg_accounts_refresh: 5u64,
                flag_fast_unlock: false,

                arg_ui_path: "$HOME/.parity/signer".into(),

                // -- Networking Options
                flag_no_warp: false,
                arg_port: 30303u16,
                arg_interface: "all".into(),
                arg_min_peers: Some(25u16),
                arg_max_peers: Some(50u16),
                arg_max_pending_peers: 64u16,
                arg_snapshot_peers: 0u16,
                arg_allow_ips: "all".into(),
                arg_nat: "any".into(),
                arg_network_id: Some(1),
                arg_bootnodes: Some("".into()),
                flag_no_discovery: false,
                arg_node_key: None,
                arg_reserved_peers: Some("./path_to_file".into()),
                flag_reserved_only: false,
                flag_no_ancient_blocks: false,
                arg_warp_barrier: None,

                // -- API and Console Options
                // RPC
                flag_no_jsonrpc: false,
                flag_jsonrpc_no_keep_alive: false,
                flag_jsonrpc_experimental: false,
                arg_jsonrpc_port: 8545u16,
                arg_jsonrpc_interface: "local".into(),
                arg_jsonrpc_cors: "null".into(),
                arg_jsonrpc_apis: "web3,eth,net,parity,traces,secretstore".into(),
                arg_jsonrpc_hosts: "none".into(),
                arg_jsonrpc_server_threads: None,
                arg_jsonrpc_threads: 4,
                arg_jsonrpc_max_payload: None,
                arg_poll_lifetime: 60u32,
                flag_jsonrpc_allow_missing_blocks: false,

                // WS
                flag_no_ws: false,
                arg_ws_port: 8546u16,
                arg_ws_interface: "local".into(),
                arg_ws_apis: "web3,eth,net,parity,traces,secretstore".into(),
                arg_ws_origins: "none".into(),
                arg_ws_hosts: "none".into(),
                arg_ws_max_connections: 100,

                // IPC
                flag_no_ipc: false,
                arg_ipc_path: "$HOME/.parity/jsonrpc.ipc".into(),
                arg_ipc_apis: "web3,eth,net,parity,parity_accounts,personal,traces,secretstore"
                    .into(),

                // METRICS
                flag_metrics: false,
                arg_metrics_port: 3000u16,
                arg_metrics_interface: "local".into(),

                // SECRETSTORE
                flag_no_secretstore: false,
                flag_no_secretstore_http: false,
                flag_no_secretstore_auto_migrate: false,
                arg_secretstore_acl_contract: Some("registry".into()),
                arg_secretstore_contract: Some("none".into()),
                arg_secretstore_srv_gen_contract: Some("none".into()),
                arg_secretstore_srv_retr_contract: Some("none".into()),
                arg_secretstore_doc_store_contract: Some("none".into()),
                arg_secretstore_doc_sretr_contract: Some("none".into()),
                arg_secretstore_secret: None,
                arg_secretstore_admin_public: None,
                arg_secretstore_nodes: "".into(),
                arg_secretstore_server_set_contract: Some("registry".into()),
                arg_secretstore_interface: "local".into(),
                arg_secretstore_port: 8083u16,
                arg_secretstore_http_interface: "local".into(),
                arg_secretstore_http_port: 8082u16,
                arg_secretstore_path: "$HOME/.parity/secretstore".into(),

                // -- Sealing/Mining Options
                arg_author: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
                arg_engine_signer: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
                flag_force_sealing: true,
                arg_reseal_on_txs: "all".into(),
                arg_reseal_min_period: 4000u64,
                arg_reseal_max_period: 60000u64,
                flag_reseal_on_uncle: false,
                arg_work_queue_size: 20usize,
                arg_tx_gas_limit: Some("10000000".into()),
                arg_tx_time_limit: Some(100u64),
                arg_relay_set: "cheap".into(),
                arg_min_gas_price: Some(0u64),
                arg_usd_per_tx: "0.0001".into(),
                arg_gas_price_percentile: 50usize,
                arg_usd_per_eth: "auto".into(),
                arg_price_update_period: "hourly".into(),
                arg_gas_floor_target: "8000000".into(),
                arg_gas_cap: "10000000".into(),
                arg_extra_data: Some("Parity".into()),
                flag_tx_queue_no_unfamiliar_locals: false,
                flag_tx_queue_no_early_reject: false,
                arg_tx_queue_size: 8192usize,
                arg_tx_queue_per_sender: None,
                arg_tx_queue_mem_limit: 4u32,
                arg_tx_queue_locals: Some("0xdeadbeefcafe0000000000000000000000000000".into()),
                arg_tx_queue_strategy: "gas_factor".into(),
                flag_remove_solved: false,
                arg_notify_work: Some("http://localhost:3001".into()),
                flag_refuse_service_transactions: false,
                flag_infinite_pending_block: false,
                arg_max_round_blocks_to_import: 12usize,

                flag_stratum: false,
                arg_stratum_interface: "local".to_owned(),
                arg_stratum_port: 8008u16,
                arg_stratum_secret: None,

                // -- Footprint Options
                arg_tracing: "auto".into(),
                arg_pruning: "auto".into(),
                arg_pruning_history: 64u64,
                arg_pruning_memory: 500usize,
                arg_cache_size_db: 64u32,
                arg_cache_size_blocks: 8u32,
                arg_cache_size_queue: 50u32,
                arg_cache_size_state: 25u32,
                arg_cache_size: Some(128),
                arg_db_compaction: "ssd".into(),
                arg_fat_db: "auto".into(),
                flag_scale_verifiers: true,
                arg_num_verifiers: Some(6),

                // -- Import/Export Options
                arg_export_blocks_from: "1".into(),
                arg_export_blocks_to: "latest".into(),
                flag_no_seal_check: false,
                flag_export_state_no_code: false,
                flag_export_state_no_storage: false,
                arg_export_state_min_balance: None,
                arg_export_state_max_balance: None,

                // -- Snapshot Optons
                arg_export_state_at: "latest".into(),
                arg_snapshot_at: "latest".into(),
                flag_enable_snapshotting: false,
                arg_snapshot_threads: None,

                // -- Internal Options
                flag_can_restart: false,

                // -- Miscellaneous Options
                flag_version: false,
                arg_logging: Some("own_tx=trace".into()),
                arg_log_file: Some("/var/log/openethereum.log".into()),
                flag_no_color: false,
                flag_no_config: false,
            }
        );
    }

    #[test]
    fn should_parse_config_and_return_errors() {
        let config1 = Args::parse_config(include_str!("./tests/config.invalid1.toml"));
        let config2 = Args::parse_config(include_str!("./tests/config.invalid2.toml"));
        let config3 = Args::parse_config(include_str!("./tests/config.invalid3.toml"));
        let config4 = Args::parse_config(include_str!("./tests/config.invalid4.toml"));

        match (config1, config2, config3, config4) {
            (
                Err(ArgsError::Decode(_)),
                Err(ArgsError::Decode(_)),
                Err(ArgsError::Decode(_)),
                Err(ArgsError::Decode(_)),
            ) => {}
            (a, b, c, d) => {
                assert!(
                    false,
                    "Got invalid error types: {:?}, {:?}, {:?}, {:?}",
                    a, b, c, d
                );
            }
        }
    }

    #[test]
    fn should_deserialize_toml_file() {
        let config: Config = toml::from_str(include_str!("./tests/config.toml")).unwrap();

        assert_eq!(
            config,
            Config {
                parity: Some(Operating {
                    mode: Some("dark".into()),
                    mode_timeout: Some(15u64),
                    mode_alarm: Some(10u64),
                    chain: Some("./chain.json".into()),
                    base_path: None,
                    db_path: None,
                    keys_path: None,
                    identity: None,
                    no_persistent_txqueue: None,
                }),
                account: Some(Account {
                    unlock: Some(vec!["0x1".into(), "0x2".into(), "0x3".into()]),
                    password: Some(vec!["passwdfile path".into()]),
                    keys_iterations: None,
                    refresh_time: None,
                    fast_unlock: None,
                }),
                ui: None,
                network: Some(Network {
                    warp: Some(false),
                    warp_barrier: None,
                    port: None,
                    interface: None,
                    min_peers: Some(10),
                    max_peers: Some(20),
                    max_pending_peers: Some(30),
                    snapshot_peers: Some(40),
                    allow_ips: Some("public".into()),
                    nat: Some("any".into()),
                    id: None,
                    bootnodes: None,
                    discovery: Some(true),
                    node_key: None,
                    reserved_peers: Some("./path/to/reserved_peers".into()),
                    reserved_only: Some(true),
                }),
                websockets: Some(Ws {
                    disable: Some(true),
                    port: None,
                    interface: None,
                    apis: None,
                    origins: Some(vec!["none".into()]),
                    hosts: None,
                    max_connections: None,
                }),
                rpc: Some(Rpc {
                    disable: Some(true),
                    port: Some(8180),
                    interface: None,
                    cors: None,
                    apis: None,
                    hosts: None,
                    server_threads: None,
                    processing_threads: None,
                    max_payload: None,
                    keep_alive: None,
                    experimental_rpcs: None,
                    poll_lifetime: None,
                    allow_missing_blocks: None
                }),
                ipc: Some(Ipc {
                    disable: None,
                    path: None,
                    apis: Some(vec!["rpc".into(), "eth".into()]),
                }),
                metrics: Some(Metrics {
                    enable: Some(true),
                    interface: Some("local".to_string()),
                    port: Some(4000),
                }),
                secretstore: Some(SecretStore {
                    disable: None,
                    disable_http: None,
                    disable_auto_migrate: None,
                    acl_contract: None,
                    service_contract: None,
                    service_contract_srv_gen: None,
                    service_contract_srv_retr: None,
                    service_contract_doc_store: None,
                    service_contract_doc_sretr: None,
                    self_secret: None,
                    admin_public: None,
                    nodes: None,
                    server_set_contract: None,
                    interface: None,
                    port: Some(8083),
                    http_interface: None,
                    http_port: Some(8082),
                    path: None,
                }),
                mining: Some(Mining {
                    author: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
                    engine_signer: Some("0xdeadbeefcafe0000000000000000000000000001".into()),
                    force_sealing: Some(true),
                    reseal_on_txs: Some("all".into()),
                    reseal_on_uncle: None,
                    reseal_min_period: Some(4000),
                    reseal_max_period: Some(60000),
                    work_queue_size: None,
                    relay_set: None,
                    min_gas_price: None,
                    gas_price_percentile: None,
                    usd_per_tx: None,
                    usd_per_eth: None,
                    price_update_period: Some("hourly".into()),
                    gas_floor_target: None,
                    gas_cap: None,
                    tx_queue_size: Some(8192),
                    tx_queue_per_sender: None,
                    tx_queue_mem_limit: None,
                    tx_queue_locals: None,
                    tx_queue_strategy: None,
                    tx_queue_ban_count: None,
                    tx_queue_ban_time: None,
                    tx_queue_no_unfamiliar_locals: None,
                    tx_queue_no_early_reject: None,
                    tx_gas_limit: None,
                    tx_time_limit: None,
                    extra_data: None,
                    remove_solved: None,
                    notify_work: None,
                    refuse_service_transactions: None,
                    infinite_pending_block: None,
                    max_round_blocks_to_import: None,
                }),
                footprint: Some(Footprint {
                    tracing: Some("on".into()),
                    pruning: Some("fast".into()),
                    pruning_history: Some(64),
                    pruning_memory: None,
                    fast_and_loose: None,
                    cache_size: None,
                    cache_size_db: Some(256),
                    cache_size_blocks: Some(16),
                    cache_size_queue: Some(100),
                    cache_size_state: Some(25),
                    db_compaction: Some("ssd".into()),
                    fat_db: Some("off".into()),
                    scale_verifiers: Some(false),
                    num_verifiers: None,
                }),
                snapshots: Some(Snapshots {
                    enable: Some(false),
                    processing_threads: None,
                }),
                misc: Some(Misc {
                    logging: Some("own_tx=trace".into()),
                    log_file: Some("/var/log/openethereum.log".into()),
                    color: Some(true),
                    ports_shift: Some(0),
                    unsafe_expose: Some(false),
                }),
                stratum: None,
            }
        );
    }

    #[test]
    fn should_not_accept_min_peers_bigger_than_max_peers() {
        match Args::parse(&["openethereum", "--max-peers=39", "--min-peers=40"]) {
            Err(ArgsError::PeerConfiguration) => (),
            _ => assert_eq!(false, true),
        }
    }

    #[test]
    fn should_accept_max_peers_equal_or_bigger_than_min_peers() {
        Args::parse(&["openethereum", "--max-peers=40", "--min-peers=40"]).unwrap();
        Args::parse(&["openethereum", "--max-peers=100", "--min-peers=40"]).unwrap();
    }
}
