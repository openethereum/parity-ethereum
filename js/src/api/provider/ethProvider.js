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

import Provider from './provider';

export default class EthProvider extends Provider {
  constructor (transport) {
    super(transport);
    this._api = 'eth';
  }

  unsubscribe (...subscriptionIds) {
    return this._removeListener('eth', subscriptionIds);
  }

  // `newHeads`, `logs`, `newPendingTransactions`, `syncing`
  newHeads (callback) {
    return this._addListener(this._api, 'newHeads', callback);
  }

  logs (callback) {
    // not supported yet
    throw Error('not supported yet');
    // return this._addListener(this._api, 'logs', callback);
  }

  newPendingTransactions (callback) {
    // not supported yet
    throw Error('not supported yet');
    // return this._addListener(this._api, 'newPendingTransactions', callback);
  }

  syncing (callback) {
    // not supported yet
    throw Error('not supported yet');
    // return this._addListener(this._api, 'syncing', callback);
  }
}
