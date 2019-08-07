#include "websocket_subscription.hpp"
#include "parity.h"

WebsocketSubscription::WebsocketSubscription(const void *inner): inner(inner) {}

WebsocketSubscription::~WebsocketSubscription() {
	if (inner) {
		parity_unsubscribe_ws(inner);
		inner = nullptr;
	}
}
