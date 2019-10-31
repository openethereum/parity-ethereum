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

/// The C interface to Parity Ethereum.
///
/// # Thread safety
///
/// The Parity Ethereum C API is thread safe. All resources can be operated on
/// by multiple threads simultaneously.
///
/// Your callbacks are also expected to be thread safe. Parity Ethereum makes
/// heavy use of background threads internally, rather than using a
/// user-provided event loop. Therefore, your callbacks may be called
/// from any thread, or even from multiple threads at once. They need to be
/// prepared for that. A good way to handle this is for your callbacks to
/// deserialize the message and then use a thread-safe queue to deliver the
/// message to your event loop.
///
/// # Blocking
///
/// The Parity Ethereum C API generally does not block on network I/O, except
/// when a Parity Ethereum instance is being destroyed. However, it may block on
/// disk I/O at any time. If this presents a problem in your application, you
/// should call the Parity Ethereum C API from a worker thread that is allowed
/// to block.
///
/// Your callbacks should also not block under normal circumstances. They do not
/// need to be real-time, but you are expected to be able to keep up with the
/// events you have subscribed to. The blockchain won’t slow down for you, so if
/// you cannot keep pace with incoming messages, you will fall behind.
///
/// # Constructing multiple Parity Ethereum instances
///
/// Constructing multiple instances of Parity Ethereum is discouraged. While it
/// is expected to work, they will duplicate a large amount of state
#ifndef PARITY_H_INCLUDED
#define PARITY_H_INCLUDED 1
#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>

/// An opaque struct that represents a Parity configuration.
struct parity_config;

/// An opaque struct that represents a Parity instance.
struct parity_ethereum;

/// An opaque struct that represents a Parity logger
struct parity_logger;

/// An opaque struct that represents a Parity subscription
struct parity_subscription;

#if defined configuration || defined logger || defined custom ||               \
    defined new_chain || defined new_chain_len ||                              \
    defined on_client_restart_cb || defined args || defined len ||             \
    defined arg_lens || defined parity_subscription ||                         \
    defined parity_logger || defined parity_config
#error macro conflicts with Parity Ethereum C API header
#endif

typedef void (*parity_destructor)(void *);

/// Parameters to pass to `parity_start`.
struct ParityParams {
  /// Configuration object, as handled by the `parity_config_*` functions.
  /// Note that calling `parity_start` will destroy the configuration object
  /// (even on failure).
  struct parity_config *configuration;

  /// Callback function to call when the client receives an RPC request to
  /// change its chain spec.
  ///
  /// Will only be called if you enable the `--can-restart` flag.
  ///
  /// The first parameter of the callback is the value of
  /// `on_client_restart_cb_custom`. The second and third parameters of the
  /// callback are the string pointer and length.
  void (*on_client_restart_cb)(void *custom, const char *new_chain,
                               uintptr_t new_chain_len);

  /// Custom parameter passed to the `on_client_restart_cb` callback as first
  /// parameter.
  void *on_client_restart_cb_custom;

  /// Callback for when a client is destroyed.
  /// If this is NULL, no callback is called.
  parity_destructor on_client_destroy;

  /// Logger object which must be created by the `parity_config_logger` function
  struct parity_logger *logger;
};

/// Builds a new configuration object by parsing a list of CLI arguments.
///
/// The first two parameters are string pointers and string lengths. They must
/// have a length equal to `len`. The strings don't need to be zero-terminated.
///
/// On success, the produced object will be written to the `struct parity_config
/// *` pointed by `out`.
///
/// Returns 0 on success, and non-zero on error.
///
/// # Example
///
/// ```no_run
/// void* cfg;
/// const char *args[] = {"--light", "--can-restart"};
/// uintptr_t str_lens[] = {7, 13};
/// if (parity_config_from_cli(args, str_lens, 2, &cfg) != 0) {
/// 		return 1;
/// }
/// ```
///
bool parity_config_from_cli(char const *const *args, uintptr_t const *arg_lens,
                            uintptr_t len, struct parity_config **out);

/// Builds a new logger object to be used as a member of `struct ParityParams`.
///
/// @param log_mode String representing the log mode according to `RUST_LOG`, or
/// `nullptr` to disable logging. See module documentation for `ethcore-logger`
/// for more info.
/// @param log_mode_len Length of `log_mode`, or zero to disable logging.
/// @param log_file String respresenting the file name to write to log to, or
/// nullptr to disable logging to a file.
///                   On Windows, this will be interpreted as UTF-8, not the
///                   system codepage, and is not limited to MAX_PATH.
/// @param log_mode_len Length of the log_file or zero to disable logging to a
/// file.
/// logger Pointer to point to the created `Logger` object

/// **Important**: This function must only be called exactly once otherwise it
/// will panic. If you want to disable a logging mode or logging to a file make
/// sure that you pass the `length` as zero
///
/// # Example
///
/// ```no_run
/// void* cfg;
/// const char *args[] = {"--light", "--can-restart"};
/// uintptr_t str_lens[] = {7, 13};
/// if (parity_config_from_cli(args, str_lens, 2, &cfg) != 0) {
/// 		return 1;
/// }
/// char[] logger_mode = "rpc=trace";
/// parity_set_logger(logger_mode, strlen(logger_mode), nullptr, 0,
/// &cfg.logger);
/// ```
///
void parity_set_logger(const char *log_mode, uintptr_t log_mode_len,
                       const char *log_file, uintptr_t log_file_len,
                       struct parity_logger **logger);

/// Destroys a configuration object created earlier.
///
/// **Important**: You probably don't need to call this function. Calling
/// `parity_start` destroys the configuration object as well (even on failure).
///
/// It is safe to pass NULL here, in which case this function has no effect.
void parity_config_destroy(struct parity_config *cfg);

/// Starts the parity client in a background thread.
///
/// **Important**: The configuration object passed inside `cfg` is destroyed
/// when you call `parity_start` (even on failure).
///
/// On success, the produced object will be written to the `void*` pointed by
/// `out`.  If the provided command-line arguments didn’t require starting a
/// node, `*out` will be unchanged.
///
/// Returns 0 on success, and non-zero on error.
int parity_start(const struct ParityParams *params,
                 struct parity_ethereum **out);

/// Destroys the parity client created with `parity_start`.
///
/// If `parity` is NULL, this is a harmless no-op.
void parity_destroy(struct parity_ethereum *const parity);

/// Performs an asynchronous RPC request running in a background thread for at
/// most X milliseconds
///
/// @param parity Reference to the running parity client
/// @param rpc_query JSON encoded string representing the RPC request.
/// Parity Ethereum will make a copy of this string, so you don’t need to.
/// @param rpc_len Length of the RPC query
/// @param timeout_ms Maximum time that request is waiting for a response
/// @param subscribe Callback to invoke when the query gets answered. It will be
/// called on a background thread with a JSON encoded string with the result
/// both on success and on error.
/// @param destructor Called when `ud` is no longer in use.
/// @param ud Specific user defined data that can used in
/// the callback.
/// @return false on success, true on error.
int parity_rpc(const struct parity_ethereum *const parity,
               const char *rpc_query, uintptr_t rpc_len, uintptr_t timeout_ms,
               void (*subscribe)(void *ud, const char *response, uintptr_t len),
               parity_destructor destructor, void *ud);

/// Subscribes to a specific websocket event that will run until it is canceled
///
/// @param parity	Reference to the running parity client
/// @param ws_query JSON encoded string representing the websocket event to
/// subscribe to.
/// @param len Length of the query.
/// @param response Callback to invoke when a websocket event occurs.
/// @param ud Specific user defined data that can used in the callback.
/// @param destructor Will be called when `ud` is guaranteed to not be needed
/// again.
/// @return A handle to the subscription on success, or NULL on error.
/// The handle can be used to cancel the subscription.
struct parity_subscription *parity_subscribe_ws(
    const struct parity_ethereum *const parity, const char *ws_query,
    uintptr_t len,
    void (*subscribe)(void *ud, const char *response, uintptr_t len), void *ud,
    parity_destructor destructor);

/// Unsubscribes from a websocket subscription. This function destroys the
/// session object, leaving `session` a dangling pointer.
///
/// @param session Pointer to the session to unsubscribe from.  If `session` is
/// NULL, this is a harmless no-op.
void parity_unsubscribe_ws(const struct parity_subscription *const session);

/// Sets a callback to call when a panic happens in the Rust code.
///
/// The callback takes as parameter the `void *param` passed to this function
/// and the panic message. You are expected to log the panic message and exit.
/// If this function returns, Parity Ethereum will exit the process itself.
///
/// It is not possible to recover from a panic.  Calling `longjmp` or throwing a
/// C++ exception results in undefined behavior.  It *is* permissible to block
/// for an arbitrary amount of time in this callback.  Due to Rust’s memory
/// safety, it is unlikely that memory has been corrupted, so you can (and
/// should) save data to disk before exiting.
///
/// A panic almost always indicates a bug in Parity Etherium, and should be
/// presumed to be such unless proven otherwise.  Very rarely, a panic can
/// result from a fatal problem with the system Parity Ethereum is running on,
/// such as errors accessing the local file system, corruption of Parity
/// Ethereum’s database, or your code corrupting Parity’s memory. Nevertheless,
/// a panic is still a bug in Parity Ethereum unless proven otherwise.
///
/// Note that this method sets the panic hook for the whole program, and not
/// just for Parity. In other words, if you use multiple Rust libraries at once
/// (and not just Parity), then a panic in any Rust code will call this callback
/// as well.
///
/// ## Thread safety
///
/// The callback can be called from any thread and multiple times
/// simultaneously. Make sure that your code is thread safe.
void parity_set_panic_hook(void (*cb)(void *param, const char *msg,
                                      uintptr_t msg_len),
                           void *param);
#ifdef __cplusplus
}
#endif
#endif // include guard
