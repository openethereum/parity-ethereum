// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

export default class Provider {
  // Provider for websocket pubsub transport
  constructor (transport) {
    this._transport = transport;
  }

  addListener (module, eventName, callback, eventParams) {
    return this._addListener(module, eventName, callback, eventParams);
  }

  removeListener (module, subscriptionIds) {
    return this._removeListener(module, subscriptionIds);
  }

  _addListener (module, eventName, callback, eventParams) {
    // eth_subscribe does not support empty array as params
    return this._transport.subscribe(this._defineModule(module), callback, eventName, eventParams);
  }

  _removeListener (module, subscriptionIds) {
    return this._transport.unsubscribe(this._defineModule(module), subscriptionIds);
  }

  _defineModule (module) {
    return module === 'eth' ? 'eth_subscribe' : 'parity_subscribe';
  }
}
