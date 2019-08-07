#include "parity.h"
#include "parity_client.hpp"
#include <cstring>
#include <string>
#include <stdexcept>

ParityClient::ParityClient(std::vector<const char*> config, std::string logger_mode, std::string log_file) {
		ParityParams cfg;

		cfg.configuration = nullptr;
		cfg.on_client_restart_cb = nullptr;
		cfg.on_client_restart_cb_custom = nullptr;
		cfg.logger = nullptr;

		std::vector<size_t> str_lens;

		for (auto arg: config) {
			str_lens.push_back(std::strlen(arg));
		}

		if (config.size() > 0) {
			if (parity_config_from_cli(&config[0], &str_lens[0], config.size(), &cfg.configuration) != 0) {
				throw std::runtime_error("ParityClient config failed");
			}
		} else {
			if (parity_config_from_cli(nullptr, nullptr, 0, &cfg.configuration) != 0) {
				throw std::runtime_error("ParityClient config failed");
			}
		}

		parity_set_logger(
				logger_mode.c_str(),
				logger_mode.length(),
				log_file.c_str(),
				log_file.length(),
				&cfg.logger
		);

		void *parity = nullptr;
		if (parity_start(&cfg, &parity) != 0) {
			throw std::runtime_error("ParityClient could not be started");
		}

		inner = parity;
}

ParityClient::~ParityClient() {
	parity_destroy(inner);
}

void ParityClient::rpc_query(
	std::string query,
	void (*callback)(void* user_data, const char* response, size_t len),
	uint64_t timeout_as_millis,
	void *user_data
) const {
	if (parity_rpc(inner, query.c_str(), query.length(), timeout_as_millis, callback, user_data) != 0) {
		throw std::runtime_error("ParityClient rpc query failed");
	}
}

WebsocketSubscription ParityClient::websocket_subscribe(
		std::string event,
		void (*callback)(void* ud, const char* response, size_t len),
		void *user_data
) const {
	const void *session = parity_subscribe_ws(inner, event.c_str(), event.length(), callback, user_data);

	if (session == nullptr) {
		throw std::runtime_error("ParityClient subscription to Websocket failed");
	}

	return WebsocketSubscription {session};
}
