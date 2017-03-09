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

import { isEqual } from 'lodash';
import { action, observable } from 'mobx';

export default class SignerStore {
  @observable balances = {};
  @observable localHashes = [];

  externalLink = '';

  constructor (api, withLocalTransactions = false, externalLink = '') {
    this._api = api;
    this._timeoutId = 0;
    this.externalLink = externalLink;

    if (withLocalTransactions) {
      this.fetchLocalTransactions();
    }
  }

  @action setBalance = (address, balance) => {
    this.setBalances({ [address]: balance });
  }

  @action setBalances = (balances) => {
    this.balances = Object.assign({}, this.balances, balances);
  }

  @action setLocalHashes = (localHashes = []) => {
    // Use slice to make sure they are both Arrays (MobX uses Objects for Observable Arrays)
    if (!isEqual(localHashes.slice(), this.localHashes.slice())) {
      this.localHashes = localHashes;
    }
  }

  @action unsubscribe () {
    if (this._timeoutId) {
      clearTimeout(this._timeoutId);
    }
  }

  fetchBalance (address) {
    this._api.eth
      .getBalance(address)
      .then((balance) => {
        this.setBalance(address, balance);
      })
      .catch((error) => {
        console.warn('Store:fetchBalance', error);
      });
  }

  fetchBalances (_addresses) {
    const addresses = _addresses.filter((address) => address) || [];

    if (!addresses.length) {
      return;
    }

    Promise
      .all(addresses.map((address) => this._api.eth.getBalance(address)))
      .then((_balances) => {
        this.setBalances(
          addresses.reduce((balances, address, index) => {
            balances[address] = _balances[index];
            return balances;
          }, {})
        );
      })
      .catch((error) => {
        console.warn('Store:fetchBalances', error);
      });
  }

  fetchLocalTransactions = () => {
    const nextTimeout = () => {
      this._timeoutId = setTimeout(this.fetchLocalTransactions, 1500);
    };

    this._api.parity
      .localTransactions()
      .then((localTransactions) => {
        this.setLocalHashes(Object.keys(localTransactions));
      })
      .then(nextTimeout)
      .catch(nextTimeout);
  }
}
