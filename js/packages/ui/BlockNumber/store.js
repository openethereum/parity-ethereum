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

import { action, observable } from 'mobx';

export default class Store {
  @observable blockNumber = null;
  @observable blockTimestamp = null;
  @observable syncing = null;

  constructor (api) {
    this._api = api;
    this._api.on('connected', this.setupSubscriptions, this);

    // Connected and/or events NOT available
    if (this._api.isConnected) {
      this.setupSubscriptions();
    }
  }

  setupSubscriptions = () => {
    this._api.pubsub.eth.syncing((error, syncing) => {
      if (!error) {
        this.setSyncing(syncing);
      }
    });

    this._api.pubsub.eth.blockNumber((error, blockNumber) => {
      if (!error) {
        this.setBlockNumber(blockNumber);
      }

      this._api.parity
        .getBlockHeaderByNumber(blockNumber)
        .then((block) => {
          if (!block) {
            return;
          }

          this.setBlockTimestamp(block.timestamp);
        });
    });
  }

  @action setBlockNumber = (blockNumber) => {
    this.blockNumber = blockNumber;
  }

  @action setBlockTimestamp = (blockTimestamp) => {
    this.blockTimestamp = blockTimestamp;
  }

  @action setSyncing = (syncing) => {
    this.syncing = syncing;
  }

  static instance = null;

  static get (api) {
    if (!Store.instance) {
      Store.instance = new Store(api);
    }

    return Store.instance;
  }
}
