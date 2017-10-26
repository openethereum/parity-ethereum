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

import Eth from './eth';
import Parity from './parity';
import Net from './net';

import { isFunction } from '../util/types';

export default class Pubsub {
  constructor (transport) {
    if (!transport || !isFunction(transport.subscribe)) {
      throw new Error('Pubsub API needs transport with subscribe() function defined. (WebSocket)');
    }

    this._eth = new Eth(transport);
    this._net = new Net(transport);
    this._parity = new Parity(transport);
  }

  get net () {
    return this._net;
  }

  get eth () {
    return this._eth;
  }

  get parity () {
    return this._parity;
  }

  unsubscribe (subscriptionIds) {
    // subscriptions are namespace independent. Thus we can simply removeListener from any.
    return this._parity.removeListener(subscriptionIds);
  }
}
