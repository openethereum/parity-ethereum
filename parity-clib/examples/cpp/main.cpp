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
#include <parity.h>
#include <regex>
#include <string>
#include <thread>

void* parity_run(std::vector<const char*>);
int parity_subscribe_to_websocket(void*);
int parity_rpc_queries(void*);

const int SUBSCRIPTION_ID_LEN = 18;
const size_t TIMEOUT_ONE_MIN_AS_MILLIS = 60 * 1000;
const unsigned int CALLBACK_RPC = 1;
const unsigned int CALLBACK_WS = 2;

struct Callback {
	unsigned int type;
	long unsigned int counter;
};

// list of rpc queries
const std::vector<std::string> rpc_queries {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getTransactionReceipt\",\"params\":[\"0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// list of subscriptions
const std::vector<std::string> ws_subscriptions {
	"{\"method\":\"parity_subscribe\",\"params\":[\"eth_getBalance\",[\"0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826\",\"latest\"]],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"parity_subscribe\",\"params\":[\"parity_netPeers\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// callback that gets invoked upon an event
void callback(void* user_data, const char* response, size_t _len) {
	Callback* cb = static_cast<Callback*>(user_data);
	if (cb->type == CALLBACK_RPC) {
		printf("rpc response: %s\r\n", response);
		cb->counter -= 1;
	} else if (cb->type == CALLBACK_WS) {
		printf("websocket response: %s\r\n", response);
		std::regex is_subscription ("\\{\"jsonrpc\":\"2.0\",\"result\":\"0[xX][a-fA-F0-9]{16}\",\"id\":1\\}");
		if (std::regex_match(response, is_subscription) == true) {
			cb->counter -= 1;
		}
	}
}

int main() {
	// run full-client
	{
		std::vector<const char*> config = {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "kovan"};
		void* parity = parity_run(config);
		if (parity_rpc_queries(parity)) {
			printf("rpc_queries failed\r\n");
			return 1;
		}

		if (parity_subscribe_to_websocket(parity)) {
			printf("ws_queries failed\r\n");
			return 1;
		}

		if (parity != nullptr) {
			parity_destroy(parity);
		}
	}

	// run light-client
	{
		std::vector<const char*> light_config = {"--no-ipc", "--light", "--jsonrpc-apis=all", "--chain", "kovan"};
		void* parity = parity_run(light_config);

		if (parity_rpc_queries(parity)) {
			printf("rpc_queries failed\r\n");
			return 1;
		}

		if (parity_subscribe_to_websocket(parity)) {
			printf("ws_queries failed\r\n");
			return 1;
		}

		if (parity != nullptr) {
			parity_destroy(parity);
		}
	}
	return 0;
}

int parity_rpc_queries(void* parity) {
	if (!parity) {
		return 1;
	}

	Callback cb { .type = CALLBACK_RPC, .counter = rpc_queries.size() };

	for (auto query : rpc_queries) {
		if (parity_rpc(parity, query.c_str(), query.length(), TIMEOUT_ONE_MIN_AS_MILLIS, callback, &cb) != 0) {
			return 1;
		}
	}

	while(cb.counter != 0);
	return 0;
}


int parity_subscribe_to_websocket(void* parity) {
	if (!parity) {
		return 1;
	}

	std::vector<const void*> sessions;

	Callback cb { .type = CALLBACK_WS, .counter = ws_subscriptions.size() };

	for (auto sub : ws_subscriptions) {
		void *const session = parity_subscribe_ws(parity, sub.c_str(), sub.length(), callback, &cb);
		if (!session) {
			return 1;
		}
		sessions.push_back(session);
	}

	while(cb.counter != 0);
	std::this_thread::sleep_for(std::chrono::seconds(60));
	for (auto session : sessions) {
		parity_unsubscribe_ws(session);
	}
	return 0;
}

void* parity_run(std::vector<const char*> args) {
	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = callback,
		.on_client_restart_cb_custom = nullptr
	};

	std::vector<size_t> str_lens;

	for (auto arg: args) {
		str_lens.push_back(std::strlen(arg));
	}

	// make sure no out-of-range access happens here
	if (args.empty()) {
		if (parity_config_from_cli(nullptr, nullptr, 0, &cfg.configuration) != 0) {
			return nullptr;
		}
	} else {
		if (parity_config_from_cli(&args[0], &str_lens[0], args.size(), &cfg.configuration) != 0) {
			return nullptr;
		}
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return nullptr;
	}

	return parity;
}
