#include <mutex>
#include <regex>
#include "parity_callback.hpp"

std::regex is_websocket_subscription {"\\{\"jsonrpc\":\"2.0\",\"result\":\"0[xX][a-fA-F0-9]{16}\",\"id\":1\\}"};

namespace {
	static uint64_t counter = 0;
	std::mutex lock;
}

void ParityCallback::callback(void *user_data, const char* response, size_t response_len) {
	std::lock_guard<std::mutex> guard(lock);
	auto type = *static_cast<uint64_t*>(user_data);
	if (type == ParityCallback::RPC) {
		counter += 1;
	} else if (type == ParityCallback::WEBSOCKET) {
		if (std::regex_match(response, is_websocket_subscription)) {
			counter += 1;
		}
	}
}

uint64_t ParityCallback::getCount() {
	return counter;
}
