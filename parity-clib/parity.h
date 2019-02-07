// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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
	void (*on_client_restart_cb)(void* custom, const char* new_chain, size_t new_chain_len);

	/// Custom parameter passed to the `on_client_restart_cb` callback as first parameter.
	void *on_client_restart_cb_custom;

	/// Logger object which must be created by the `parity_config_logger` function
	void *logger;
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
///		return 1;
/// }
/// ```
///
int parity_config_from_cli(char const* const* args, size_t const* arg_lens, size_t len, void** out);

/// Builds a new logger object which should be a member of the `ParityParams struct`
///
///	- log_mode		: String representing the log mode according to `Rust LOG` or nullptr to disable logging.
///					  See module documentation for `ethcore-logger` for more info.
///	- log_mode_len	: Length of the log_mode or zero to disable logging
///	- log_file		: String respresenting the file name to write to log to or nullptr to disable logging to a file
///	- log_mode_len	: Length of the log_file or zero to disable logging to a file
///	- logger		: Pointer to point to the created `Logger` object

/// **Important**: This function must only be called exactly once otherwise it will panic. If you want to disable a
/// logging mode or logging to a file make sure that you pass the `length` as zero
///
/// # Example
///
/// ```no_run
/// void* cfg;
/// const char *args[] = {"--light", "--can-restart"};
/// size_t str_lens[] = {7, 13};
/// if (parity_config_from_cli(args, str_lens, 2, &cfg) != 0) {
///		return 1;
/// }
/// char[] logger_mode = "rpc=trace";
/// parity_set_logger(logger_mode, strlen(logger_mode), nullptr, 0, &cfg.logger);
/// ```
///
int parity_set_logger(const char* log_mode, size_t log_mode_len, const char* log_file, size_t log_file_len, void** logger);

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
///	- response		: Callback to invoke when the query gets answered. It will respond with a JSON encoded the string
///					  with the result both on success and error.
///	- ud			: Specific user defined data that can used in the callback
///
///	- On success	: The function returns 0
///	- On error		: The function returns 1
///
int parity_rpc(const void *const parity, const char* rpc_query, size_t rpc_len, size_t timeout_ms,
		void (*subscribe)(void* ud, const char* response, size_t len), void* ud);


/// Subscribes to a specific websocket event that will run until it is canceled
///
///	- parity		: Reference to the running parity client
///	- ws_query		: JSON encoded string representing the websocket event to subscribe to
///	- len			: Length of the query
///	- response		: Callback to invoke when a websocket event occurs
///	- ud			: Specific user defined data that can used in the callback
///
///	- On success	: The function returns an object to the current session
///					  which can be used cancel the subscription
///	- On error		: The function returns a null pointer
///
void* parity_subscribe_ws(const void *const parity, const char* ws_query, size_t len,
		void (*subscribe)(void* ud, const char* response, size_t len), void* ud);

/// Unsubscribes from a websocket subscription. Caution this function consumes the session object and must only be
/// used exactly once per session.
///
///	- session		: Pointer to the session to unsubscribe from
///
int parity_unsubscribe_ws(const void *const session);

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
int parity_set_panic_hook(void (*cb)(void* param, const char* msg, size_t msg_len), void* param);

#ifdef __cplusplus
}
#endif

#endif // include guard
