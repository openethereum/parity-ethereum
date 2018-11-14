// Copyright 2018 Parity Technologies (UK) Ltd.
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

#ifndef _PARITY_H_INCLUDED_
#define _PARITY_H_INCLUDED_

#include <stddef.h>

typedef void (subscribe)(void*, const char*, size_t);

/// Parameters to pass to `parity_start`.
struct ParityParams {
	/// Configuration object, as handled by the `parity_config_*` functions.
	/// Note that calling `parity_start` will destroy the configuration object (even on failure).
	void *configuration;

	/// Callback function to call when the client receives an RPC request to change its chain spec.
	///
	/// Will only be called if you enable the `--can-restart` flag.
	///
	/// The first parameter of the callback is the value of `on_client_restart_cb_custom`.
	/// The second and third parameters of the callback are the string pointer and length.
	subscribe *on_client_restart_cb;

	/// Custom parameter passed to the `on_client_restart_cb` callback as first parameter.
	void *on_client_restart_cb_custom;
};

#ifdef __cplusplus
extern "C" {
#endif

/// Builds a new configuration object by parsing a list of CLI arguments.
///
/// The first two parameters are string pointers and string lengths. They must have a length equal
/// to `len`. The strings don't need to be zero-terminated.
///
/// On success, the produced object will be written to the `void*` pointed by `out`.
///
/// Returns 0 on success, and non-zero on error.
///
/// # Example
///
/// ```no_run
/// void* cfg;
/// const char *args[] = {"--light", "--can-restart"};
/// size_t str_lens[] = {7, 13};
/// if (parity_config_from_cli(args, str_lens, 2, &cfg) != 0) {
///     return 1;
/// }
/// ```
///
int parity_config_from_cli(char const* const* args, size_t const* arg_lens, size_t len, void** out);

/// Destroys a configuration object created earlier.
///
/// **Important**: You probably don't need to call this function. Calling `parity_start` destroys
/// 				the configuration object as well (even on failure).
void parity_config_destroy(void* cfg);

/// Starts the parity client in background threads. Returns a pointer to a struct that represents
/// the running client. Can also return NULL if the execution completes instantly.
///
/// **Important**: The configuration object passed inside `cfg` is destroyed when you
/// 				call `parity_start` (even on failure).
///
/// On success, the produced object will be written to the `void*` pointed by `out`.
///
/// Returns 0 on success, and non-zero on error.
int parity_start(const ParityParams* params, void** out);

/// Destroys the parity client created with `parity_start`.
///
/// **Warning**: `parity_start` can return NULL if execution finished instantly, in which case you
///					must not call this function.
void parity_destroy(void* parity);

/// Performs an asynchronous RPC request running in a background thread for at most X milliseconds
///
///	- parity		: Reference to the running parity client
///	- rpc_query		: JSON encoded string representing the RPC request.
///	- len			: Length of the RPC query
///	- timeout_ms	: Maximum time that request is waiting for a response
///	- response		: Callback to invoke when the query gets answered. It will respond with
///
///						1) A JSON encoded string with the result
///						2) A string "empty", (got an empty response)
//						3) A string "timeout", (the query timed-out)
///
/// - On success	: The parity client reference and the query string were valid
/// - On error		: The parity client reference and the query string were not valid
///
int parity_rpc(void* parity, const char* rpc_query, size_t rpc_len, size_t timeout_ms, subscribe response);


/// Subscribes to a specific websocket event
/// FIXME: provide functionality to cancel a "subscription"
///
///	 - parity		: Reference to the running parity client
///	 - ws_query		: JSON encoded string representing the websocket and which event to subscribe to
///	 - len			: Length of the queury
///	 - response		: Callback to invoke when a websocket event occured
///
///  - On success	: The function returns a callback with a JSON encoded string
///  - On error		: The function returns a callback with the error (empty or timeout)
///
int parity_subscribe_ws(void* parity, const char* ws_query, size_t len, subscribe response);

/// Sets a callback to call when a panic happens in the Rust code.
///
/// The callback takes as parameter the custom param (the one passed to this function), plus the
/// panic message. You are expected to log the panic message somehow, in order to communicate it to
/// the user. A panic always indicates a bug in Parity.
///
/// Note that this method sets the panic hook for the whole program, and not just for Parity. In
/// other words, if you use multiple Rust libraries at once (and not just Parity), then a panic
/// in any Rust code will call this callback as well.
///
/// ## Thread safety
///
/// The callback can be called from any thread and multiple times simultaneously. Make sure that
/// your code is thread safe.
///
int parity_set_panic_hook(subscribe panic, void* param);

#ifdef __cplusplus
}
#endif

#endif // include guard
