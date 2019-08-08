#ifndef PARITY_CLIENT_H
#define PARITY_CLIENT_H

#include <cstdint>
#include <vector>
#include <string>
#include "websocket_subscription.hpp"

/// Wrapper class for managing low level interactions with the Parity client
class ParityClient {
	private:
		void *inner;

	public:
		// Constructor
		explicit ParityClient(std::vector<const char*> config, std::string logger_mode, std::string log_file);
		// Destructor
		~ParityClient();

		// Don't support the following defaults
		ParityClient(const ParityClient& b) = delete;
		ParityClient(ParityClient&& b) = delete;
		ParityClient &operator=(ParityClient&& b) = delete;
		ParityClient &operator=(const ParityClient& b) = delete;

		// Perform an asynchronous rpc request which invokes the callback when the request finished or timed out
		//
		// Throws an exception if the query failed
		void rpc_query(
				std::string query,
				void (*callback)(void* user_data, const char* response, size_t len),
				uint64_t timeout_as_millis,
				void *user_data
		) const;

		// Subscribe to a websocket event which invokes the callback when events of the subscription occurred.
		//
		// Returns a WebsocketSubscription object on the stack.
		// Be careful that you keep the subscription object as long as just you want to subscribe to the event.
		WebsocketSubscription websocket_subscribe(
				std::string event,
				void (*callback)(void* ud, const char* response, size_t len),
				void *user_data
		) const;
};

#endif
