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

    // Try to restart subscription if transport is closed
    this._api.transport.on('close', () => {
      if (this.isStarted) {
        this.start();
      }
    });
  }

  get isStarted () {
    return this._started;
  }

  start () {
    this._started = true;

    if (this._api.isPubSub) {
      return Promise.all([
        this._pollBlockNumber(false),
        this._api.pubsub
          .subscribeAndGetResult(
            callback => this._api.pubsub.eth.newHeads(callback),
            () => {
              return this._api.eth
                .blockNumber()
                .then(blockNumber => {
                  this.updateBlock(blockNumber);
                  return blockNumber;
                });
            }
          )
      ]);
    }

    // fallback to polling
    return this._pollBlockNumber(true);
  }

  _pollBlockNumber = (doTimeout) => {
    const nextTimeout = (timeout = 1000, forceTimeout = doTimeout) => {
      if (forceTimeout) {
        this._pollTimerId = setTimeout(() => {
          this._pollBlockNumber(doTimeout);
        }, timeout);
      }
    };

    if (!this._api.transport.isConnected) {
      nextTimeout(500, true);
      return;
    }

    return this._api.eth
      .blockNumber()
      .then((blockNumber) => {
        this.updateBlock(blockNumber);

        nextTimeout();
      })
      .catch(() => nextTimeout());
  }

  updateBlock (blockNumber) {
    if (!blockNumber.eq(this._lastBlock)) {
      this._lastBlock = blockNumber;
      this._updateSubscriptions('eth_blockNumber', null, blockNumber);
    }
  }
}
