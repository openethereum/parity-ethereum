// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

#include <chrono>
#include <iostream>
#include <string>
#include <stdexcept>
#include <thread>
#include "parity_callback.hpp"
#include "parity_client.hpp"
#include "websocket_subscription.hpp"

const uint64_t TIMEOUT_ONE_MIN_AS_MILLIS = 60 * 1000;

// rpc queries
const std::vector<std::string> rpc_queries {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// websocket subscriptions
const std::vector<std::string> ws_subscriptions {
	"{\"method\":\"parity_subscribe\",\"params\":[\"eth_getBalance\",[\"0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826\",\"latest\"]],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"parity_subscribe\",\"params\":[\"parity_netPeers\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

int main() {
	std::vector<const char*> config {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "goerli"};

	// Debug output for `rpc queries` and `websocket subscriptions`
	// Will generate a lot of noise
	std::string logger_mode {"rpc=debug,pubsub=debug"};

	// Don't write the output to a log file
	std::string log_file {};

	try {
		ParityClient client {config, logger_mode, log_file};

		// make rpc queries
		{
			uint64_t callback_kind = ParityCallback::RPC;
			for (auto query : rpc_queries) {
				client.rpc_query(query, ParityCallback::callback, TIMEOUT_ONE_MIN_AS_MILLIS, &callback_kind);
			}
			std::this_thread::sleep_for(std::chrono::seconds(60));

			if (ParityCallback::getCount() != 3) {
				return 1;
			}
		}

		// make websocket subscriptions
		{
			uint64_t callback_kind = ParityCallback::WEBSOCKET;

			auto one = client.websocket_subscribe(ws_subscriptions[0], ParityCallback::callback, &callback_kind);
			auto two = client.websocket_subscribe(ws_subscriptions[1], ParityCallback::callback, &callback_kind);
			auto three = client.websocket_subscribe(ws_subscriptions[2], ParityCallback::callback, &callback_kind);

			std::this_thread::sleep_for(std::chrono::seconds(60));

			if (ParityCallback::getCount() != 6) {
				return 1;
			}
		}

	} catch (const std::exception &err) {
		std::cerr << err.what() << std::endl;
	}

	return 0;
}
