export default class Provider {
  // Provider for websocket pubsub transport
  constructor (transport) {
    this._transport = transport;
  }

  _addListener (api, eventName, callback, ...eventParams) {
    return (api === 'eth_subscribe' && eventParams.length <= 0)
     ? this._transport.subscribe(api, callback, eventName)
     : this._transport.subscribe(this._api, callback, eventName, eventParams);
  }

  _removeListener (api, subscriptionIds) {
    return this._transport.unsubscribe(api, subscriptionIds);
  }
}
