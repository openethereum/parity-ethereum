#ifndef PARITY_CALLBACK_H
#define PARITY_CALLBACK_H

// Wrapper around `C callbacks`
// It assumes that only two callback kinds are used
// namely: `RPC` and `WebSocket`

namespace ParityCallback {

	const uint64_t RPC = 1;
	const uint64_t WEBSOCKET = 2;

	/// Callback to invoke
	void callback(void *user_data, const char* response, size_t response_len);

	/// Get number of times the callback has been invoked
	uint64_t getCount();
}

#endif
