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

int parity_light();
int parity_full();

// global variable to keep track of the received rpc responses
static int g_num_queries = 0;

// list of rpc queries
static const char* rpc_queries[] = {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getTransactionReceipt\",\"params\":[\"0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0066Dc48bb833d2B59f730F33952B3c29fE926F5\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// callback that gets invoked when the client restarts
void on_restart(void*, const char*, size_t) {}

// callback that is invoked on rpc responses
void rpc_response(void *, const char* response, size_t len) {
	printf("rpc_callback: %s \r\n", response);
	g_num_queries -= 1;
}

int main() {
	// run the list of queries in the light client
	if (parity_light() != 0) {
		printf("parity light client failed\r\n");
	}
	// run the list of queries in the full client
	if (parity_full() != 0) {
		printf("parity client failed\r\n");
	}
    return 0;
}

int parity_light() {
	int num_queries = sizeof(rpc_queries) / sizeof(rpc_queries[0]);
	g_num_queries = num_queries;

	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	const char* args[] = {"--light", "--no-ipc"};
	size_t str_lens[] = {strlen(args[0]), strlen(args[1])};

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens)/sizeof(str_lens[0]), &cfg.configuration) != 0) {
		return 1;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return 1;
	}

	for (int i = 0; i < num_queries; i++) {
		if (parity_rpc(parity, rpc_queries[i], strlen(rpc_queries[i]), rpc_response) != 0) {
			return 1;
		}
	}

	// wait until all queries have been answered
	while(g_num_queries != 0);

	if (parity != NULL) {
		parity_destroy(parity);
	}
	return 0;
}

int parity_full() {
	int num_queries = sizeof(rpc_queries) / sizeof(rpc_queries[0]);
	g_num_queries = num_queries;

	ParityParams cfg = {
		.configuration = nullptr,
		.on_client_restart_cb = on_restart,
		.on_client_restart_cb_custom = nullptr
	};

	const char* args[] = {"--no-ipc"};
	size_t str_lens[] = {strlen(args[0])};

	if (parity_config_from_cli(args, str_lens, sizeof(str_lens)/sizeof(str_lens[0]), &cfg.configuration) != 0) {
		return 1;
	}

	void *parity = nullptr;
	if (parity_start(&cfg, &parity) != 0) {
		return 1;
	}

	for (int i = 0; i < num_queries; i++) {
		if (parity_rpc(parity, rpc_queries[i], strlen(rpc_queries[i]), rpc_response) != 0) {
			return 1;
		}
	}

	// wait until all queries have been answered
	while(g_num_queries != 0);

	if (parity != NULL) {
		parity_destroy(parity);
	}
	return 0;
}
