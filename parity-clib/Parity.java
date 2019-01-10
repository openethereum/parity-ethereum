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

package io.parity.ethereum;

/**
 * Interface to the Parity client.
 */
public class Parity {
	/**
	 * Starts the Parity client with the CLI options passed as an array of strings.
	 *
	 * Each space-delimited option corresponds to an array entry.
	 * For example: `["--port", "12345"]`
	 *
	 * @param options The CLI options to start Parity with
	 */
	public Parity(String[] options) {
		long config = configFromCli(options);
		inner = build(config);
	}

	/** Performs an asynchronous RPC query by spawning a background thread that is executed until
	 *  either a response is received or the timeout has been expired.
	 *
	 * @param query           The JSON-encoded RPC query to perform
	 * @param timeoutMillis   The maximum time in milliseconds that the query will run
	 * @param callback        An instance of class which must have a instance method named `callback` that will be
	 *                        invoke when the result is ready
	 */
	public void rpcQuery(String query, long timeoutMillis, Object callback) {
		rpcQueryNative(inner, query, timeoutMillis, callback);
	}

	/** Subscribes to a specific WebSocket event that will run in a background thread until it is canceled.
	 *
	 * @param query     The JSON-encoded RPC query to perform
	 * @param callback  An instance of class which must have a instance method named `callback` that will be invoked
	 *                  when the result is ready
	 *
	 * @return A pointer to the current sessions which can be used to terminate the session later
	 */
	public long subscribeWebSocket(String query, Object callback) {
		return subscribeWebSocketNative(inner, query, callback);
	}

	/** Unsubscribes to a specific WebSocket event
	 *
	 * @param session	Pointer the the session to terminate
	 */
	public void unsubscribeWebSocket(long session) {
		unsubscribeWebSocketNative(session);
	}

	// FIXME: `finalize` is deprecated - https://github.com/paritytech/parity-ethereum/issues/10066
	@Override
	protected void finalize​() {
		destroy(inner);
	}

	static {
		System.loadLibrary("parity");
	}

	private static native long configFromCli(String[] cliOptions);
	private static native long build(long config);
	private static native void destroy(long inner);
	private static native void rpcQueryNative(long inner, String rpc, long timeoutMillis, Object callback);
	private static native long subscribeWebSocketNative(long inner, String rpc, Object callback);
	private static native void unsubscribeWebSocketNative(long session);

	private long inner;
}
