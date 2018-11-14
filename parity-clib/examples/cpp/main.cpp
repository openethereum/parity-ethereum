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

void* parity_light();
void* parity_full_run();
int parity_subscribe_to_websocket(void*);
int parity_rpc_queries(void*);

// global variable to keep track of the received rpc responses
static int g_rpc_counter = 0;

// list of websocket queries
static const char* ws_queries[] = {
	"{\"method\":\"parity_subscribe\",\"params\":[\"eth_getBalance\",[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\",\"latest\"]],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_subscribe\",\"params\":[\"newHeads\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

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
void ws_response(void *, const char* response, size_t len) {
	printf("ws_callback: %s \r\n", response);
}

// callback that is invoked on ws responses
void rpc_response(void *, const char* response, size_t len) {
	printf("rpc_callback: %s \r\n", response);
	g_rpc_counter -= 1;
}

int main() {
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
		printf("parity client was err couldn't shutdown\r\n");
		parity_destroy(parity);
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

	int num_queries = sizeof(ws_queries) / sizeof(ws_queries[0]);

	for (int i = 0; i < num_queries; i++) {
		if (parity_subscribe_ws(parity, ws_queries[i], strlen(ws_queries[i]), ws_response) != 0) {
			return 1;
		}
	}

	// wait forever
	while(1);
}

void* parity_full_run() {
	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	const char* args[] = {"--no-ipc" , "--jsonrpc-apis=all", "--chain", "kovan"};
	size_t str_lens[] = {strlen(args[0]), strlen(args[1]), strlen(args[2]), strlen(args[3])};

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens)/sizeof(str_lens[0]), &cfg.configuration) != 0) {
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

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens)/sizeof(str_lens[0]), &cfg.configuration) != 0) {
		return nullptr;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return nullptr;
	}
	return parity;
}
