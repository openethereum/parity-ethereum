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

    /** Performs a synchronous RPC query.
     *
     * Note that this will block the current thread until the query is finished. You are
     * encouraged to create a background thread if you don't want to block.
     *
     * @param query The JSON-encoded RPC query to perform
     * @return A JSON-encoded result
     */
    public String rpcQuery(String query) {
        return rpcQueryNative(inner, query);
    }

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
    private static native String rpcQueryNative(long inner, String rpc);

    private long inner;
}
