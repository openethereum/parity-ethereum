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

// Note, `nullptr` requires a C++ compiler with C+11 support

#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <unistd.h>
#include <parity.h>
#include <regex>

void* parity_light_run();
void* parity_full_run();
int parity_subscribe_to_websocket(void*);
int parity_rpc_queries(void*);

const int SUBSCRIPTION_ID_LEN = 18;

// global variable to keep track of the received rpc responses
static int g_rpc_counter = 0;

// global string for callbacks
static char g_str[60];

// list of rpc queries
static const char* rpc_queries[] = {
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
	if (std::regex_match(response, is_subscription) == true) {
		strncpy(g_str, response, 55);
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

		if (parity != NULL) {
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

		if (parity != NULL) {
			parity_destroy(parity);
		}
	}

	return 0;
}

int parity_rpc_queries(void* parity) {
	if (!parity) {
		return 1;
	}

	size_t num_queries = sizeof(rpc_queries) / sizeof(rpc_queries[0]);
	size_t timeout = 1000;
	g_rpc_counter = num_queries;


	for (int i = 0; i < num_queries; i++) {
		if (parity_rpc(parity, rpc_queries[i], strlen(rpc_queries[i]), timeout, rpc_response) != 0) {
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

	char subscribe[] = "{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}";
	char unsubscribe[] = "{\"method\":\"eth_unsubscribe\",\"params\":[\"0x1234567891234567\"],\"id\":1,\"jsonrpc\":\"2.0\"}";

	const void *const handle = parity_subscribe_ws(parity, subscribe, strlen(subscribe), ws_response);

	if (!handle) {
		return 1;
	}

	while(g_str[0] == 0);
	sleep(60);

	// Replace subscription_id with the id we got in the callback
	// (this is not a good practice use your favorite JSON parser)
	strncpy(&unsubscribe[39], &g_str[27], SUBSCRIPTION_ID_LEN);
	if (parity_unsubscribe_ws(parity, handle, unsubscribe, strlen(unsubscribe), timeout, ws_response) != 0) {
			return 1;
	}

	return 0;
}

void* parity_full_run() {
	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	const char* args[] = {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "kovan"};
	size_t str_lens[] = {strlen(args[0]), strlen(args[1]), strlen(args[2]), strlen(args[3])};

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens) / sizeof(str_lens[0]), &cfg.configuration) != 0) {
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

	const char* args[] = {"--light", "--no-ipc","--chain", "kovan", "--jsonrpc-apis=all"};
	size_t str_lens[] = {strlen(args[0]), strlen(args[1]), strlen(args[2]), strlen(args[3]), strlen(args[4])};

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens) / sizeof(str_lens[0]), &cfg.configuration) != 0) {
		return nullptr;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return nullptr;
	}
	return parity;
}
