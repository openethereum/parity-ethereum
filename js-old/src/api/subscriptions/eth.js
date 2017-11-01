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

import BigNumber from 'bignumber.js';

export default class Eth {
  constructor (updateSubscriptions, api) {
    this._api = api;
    this._updateSubscriptions = updateSubscriptions;
    this._started = false;

    this._lastBlock = new BigNumber(-1);
    this._pollTimerId = null;
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    return this._blockNumber();
  }

  _blockNumber = () => {
    const nextTimeout = (timeout = 1000) => {
      this._pollTimerId = setTimeout(() => {
        this._blockNumber();
      }, timeout);
    };

    if (!this._api.transport.isConnected) {
      nextTimeout(500);
      return;
    }

    return this._api.eth
      .blockNumber()
      .then((blockNumber) => {
        if (!blockNumber.eq(this._lastBlock)) {
          this._lastBlock = blockNumber;
          this._updateSubscriptions('eth_blockNumber', null, blockNumber);
        }

        nextTimeout();
      })
      .catch(() => nextTimeout());
  }
}
