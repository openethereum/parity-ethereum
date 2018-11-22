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

import java.util.Vector;
import java.util.concurrent.atomic.AtomicInteger;
import io.parity.ethereum.Parity;
import com.sun.jna.Pointer;

class Main {
	public static final int ONE_MINUTE_AS_MILLIS = 60 * 1000;

	public static final String[] rpc_queries = {
		"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
		"{\"method\":\"eth_getTransactionReceipt\",\"params\":[\"0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
		"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
		"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
	};

	public static final String[] ws_queries = {
		"{\"method\":\"parity_subscribe\",\"params\":[\"eth_getBalance\",[\"0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826\",\"latest\"]],\"id\":1,\"jsonrpc\":\"2.0\"}",
		"{\"method\":\"parity_subscribe\",\"params\":[\"parity_netPeers\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
		"{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
	};

	public static void runParity(String[] config) {
		Parity parity = new Parity(config);

		Callback rpcCallback = new Callback();
		Callback webSocketCallback = new Callback();

		for (String query : rpc_queries) {
			parity.rpcQuery(query, ONE_MINUTE_AS_MILLIS, rpcCallback);
		}

		while (rpcCallback.getNumCallbacks() != 4);

		Vector<Object> sessions = new Vector<Object>();

		for (String ws : ws_queries) {
			parity.subscribeWebSocket(ws, webSocketCallback);
		}

		try {
			Thread.sleep(ONE_MINUTE_AS_MILLIS);
		} catch (Exception e) {
			System.out.println(e);
		}

		for (Object session : sessions) {
			parity.unsubscribeWebSocket(session);
		}
	}

	public static void main(String[] args) {
		String[] full = {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "kovan"};
		String[] light = {"--no-ipc", "--light", "--jsonrpc-apis=all", "--chain", "kovan"};
		runParity(full);
	}
}

class Callback {
	private AtomicInteger counter;

	public Callback() {
		counter = new AtomicInteger();
	}

	public void callback(long kind, long userData, long response, long len) {
		Pointer responsePointer = new Pointer(response);
		String s = responsePointer.getString(0);
		System.out.println("Callback Kind: " + kind + s);
		counter.getAndIncrement();
	}

	public int getNumCallbacks() {
		return counter.intValue();
	}
}
