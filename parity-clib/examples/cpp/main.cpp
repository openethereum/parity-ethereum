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

#include <atomic>
#include <chrono>
#include <iostream>
#include <regex>
#include <string>
#include <stdexcept>
#include <thread>
#include "parity_client.hpp"
#include "websocket_subscription.hpp"

const uint64_t SUBSCRIPTION_ID_LEN = 18;
const uint64_t TIMEOUT_ONE_MIN_AS_MILLIS = 60 * 1000;
const uint64_t CALLBACK_RPC = 1;
const uint64_t CALLBACK_WS = 2;

std::atomic<uint64_t> callback_counter;

// list of rpc queries
const std::vector<std::string> rpc_queries {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// list of subscriptions
const std::vector<std::string> ws_subscriptions {
	"{\"method\":\"parity_subscribe\",\"params\":[\"eth_getBalance\",[\"0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826\",\"latest\"]],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"parity_subscribe\",\"params\":[\"parity_netPeers\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// Callback that gets invoked upon an event
void callback(void* user_data, const char* response, size_t _len) {
	uint64_t type = *static_cast<uint64_t*>(user_data);
	if (type == CALLBACK_RPC) {
		callback_counter += 1;
	} else if (type == CALLBACK_WS) {
		std::regex is_subscription {"\\{\"jsonrpc\":\"2.0\",\"result\":\"0[xX][a-fA-F0-9]{16}\",\"id\":1\\}"};
		if (std::regex_match(response, is_subscription)) {
			callback_counter += 1;
		}
	}
}

int main() {
	std::vector<const char*> config {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "goerli"};
	std::string logger_mode {"rpc=debug,pubsub=debug"};
	std::string log_file {};

	try {
		ParityClient client {config, logger_mode, log_file};

		// make rpc queries
		{
			uint64_t type = CALLBACK_RPC;
			callback_counter = 0;

			for (auto query : rpc_queries) {
				client.rpc_query(query, callback, TIMEOUT_ONE_MIN_AS_MILLIS, &type);
			}
			std::this_thread::sleep_for(std::chrono::seconds(60));

			if (callback_counter.load() != rpc_queries.size()) {
				return 1;
			}
		}

		// make websocket subscriptions
		{
			uint64_t type = CALLBACK_WS;
			callback_counter = 0;

			// make sure the websocket subscriptions `live` long enough
			auto one = client.websocket_subscribe(ws_subscriptions[0], callback, &type);
			auto two = client.websocket_subscribe(ws_subscriptions[1], callback, &type);
			auto three = client.websocket_subscribe(ws_subscriptions[2], callback, &type);

			std::this_thread::sleep_for(std::chrono::seconds(60));

			if (callback_counter.load() != ws_subscriptions.size()) {
				return 1;
			}
		}

	} catch (const std::exception &err) {
		std::cerr << err.what() << std::endl;
	}

	return 0;
}
