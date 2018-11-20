// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

#include <chrono>
#include <parity.h>
#include <regex>
#include <string>
#include <thread>

void* parity_light_run();
void* parity_full_run();
int parity_subscribe_to_websocket(void*);
int parity_rpc_queries(void*);

const int SUBSCRIPTION_ID_LEN = 18;

// global variable to keep track of the received rpc responses
static int g_rpc_counter = 0;

// list of rpc queries
static std::vector<std::string> rpc_queries {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getTransactionReceipt\",\"params\":[\"0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// callback that gets invoked when the client restarts
void on_restart(void*, const char*, size_t) {}

// callback that is invoked on ws responses
void ws_response(void* _unused, const char* response, size_t len) {
	printf("ws_response: %s\r\n", response);
	std::regex is_subscription ("\\{\"jsonrpc\":\"2.0\",\"result\":\"0[xX][a-fA-F0-9]{16}\",\"id\":1\\}");
	// assume only one subscription is used
	if (std::regex_match(response, is_subscription) == true) {
		g_rpc_counter -= 1;
	}
}

// callback that is invoked on ws responses
void rpc_response(void* _unused, const char* response, size_t len) {
	printf("rpc_response: %s\r\n", response);
	g_rpc_counter -= 1;
}

int main() {
	// run full-client
	{
		void* parity = parity_full_run();
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
		void* parity = parity_light_run();

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

	size_t num_queries = rpc_queries.size();
	size_t timeout = 1000;
	g_rpc_counter = num_queries;

	for (auto query : rpc_queries) {
		if (parity_rpc(parity, query.c_str(), query.length(), timeout, rpc_response) != 0) {
			return 1;
		}
	}

	while(g_rpc_counter != 0);
	return 0;
}


int parity_subscribe_to_websocket(void* parity) {
	if (!parity) {
		return 1;
	}

	size_t timeout = 1000;
	int num_queries = 1;
	g_rpc_counter = 1;

	std::string subscribe = "{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}";

	const void *const handle = parity_subscribe_ws(parity, subscribe.c_str(), subscribe.length(), ws_response);

	if (!handle) {
		return 1;
	}

	while(g_rpc_counter != 0);
	std::this_thread::sleep_for(std::chrono::seconds(60));

	parity_unsubscribe_ws(handle);
	return 0;
}

void* parity_full_run() {
	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	std::vector<const char*> args = {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "kovan"};
	std::vector<size_t> strs_len;

	for (auto arg: args) {
		strs_len.push_back(std::strlen(arg));
	}

	if (parity_config_from_cli(&args[0], &strs_len[0], args.size(), &cfg.configuration) != 0) {
		return nullptr;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return nullptr;
	}

	return parity;
}

void* parity_light_run() {
	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	std::vector<const char*> args = {"--no-ipc" , "--light", "--jsonrpc-apis=all", "--chain", "kovan"};
	std::vector<size_t> str_lens;

	for (auto arg: args) {
		str_lens.push_back(std::strlen(arg));
	}

	if (parity_config_from_cli(&args[0], &str_lens[0], str_lens.size(), &cfg.configuration) != 0) {
		return nullptr;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return nullptr;
	}
	return parity;
}
