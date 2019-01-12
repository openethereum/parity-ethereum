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

#include <cstring>
#include <parity.h>
#include <string>
#include <vector>

void* parity_run(std::vector<const char*>);
int parity_rpc_queries(void*);

const size_t TIMEOUT_THIRTY_SECS_AS_MILLIS = 30 * 1000;
const unsigned int CALLBACK_RPC = 1;

struct Callback {
	unsigned int type;
	long unsigned int counter;
};

// list of rpc queries
const std::vector<std::string> rpc_queries {
	"{\"method\":\"parity_versionInfo\",\"params\":[],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getTransactionReceipt\",\"params\":[\"0x444172bef57ad978655171a8af2cfd89baa02a97fcb773067aef7794d6913fff\"],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_estimateGas\",\"params\":[{\"from\":\"0x0010f94b296A852aAac52EA6c5Ac72e03afD032D\"}],\"id\":1,\"jsonrpc\":\"2.0\"}",
	"{\"method\":\"eth_getBalance\",\"params\":[\"0x0010f94b296A852aAac52EA6c5Ac72e03afD032D\"],\"id\":1,\"jsonrpc\":\"2.0\"}"
};

// callback that gets invoked upon an event
void callback(void* user_data, const char* response, size_t _len) {
	Callback* cb = static_cast<Callback*>(user_data);
	if (cb->type == CALLBACK_RPC) {
		printf("rpc response: %s\r\n", response);
		cb->counter -= 1;
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
		if (parity_rpc(parity, query.c_str(), query.length(), TIMEOUT_THIRTY_SECS_AS_MILLIS, callback, &cb) != 0) {
			return 1;
		}
	}

	while(cb.counter != 0);
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
