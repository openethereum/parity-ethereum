#ifndef WEBSOCKET_SUBSCRIPTION_H
#define WEBSOCKET_SUBSCRIPTION_H

// Class for managing WebSocket subscriptions
// It is a type-safe wrapper over a raw pointer
class WebsocketSubscription {
	private:
		const void *inner;

	public:
		explicit WebsocketSubscription(const void *inner);
		~WebsocketSubscription();
};

#endif
