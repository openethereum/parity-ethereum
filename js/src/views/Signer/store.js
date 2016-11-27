// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { isEqual } from 'lodash';
import { action, observable } from 'mobx';

export default class Store {
  @observable balances = {};
  @observable localTransactions = [];
  @observable doPolling = true;

  constructor (api, withLocalTransactions = false) {
    this._api = api;

    if (withLocalTransactions) {
      this.fetchLocalTransactions();
    }
  }

  @action unsubscribe () {
    this.doPolling = false;
  }

  @action setBalance = (address, balance) => {
    this.setBalances({ [address]: balance });
  }

  @action setBalances = (balances) => {
    this.balances = Object.assign({}, this.balances, balances);
  }

  @action setLocalTransactions = (localTransactions) => {
    if (!isEqual(localTransactions, this.localTransactions)) {
      this.localTransactions = localTransactions;
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
      if (this.doPolling) {
        setTimeout(this.fetchLocalTransactions, 1500);
      }
    };

    Promise
      .all([
        this._api.parity.pendingTransactions(),
        this._api.parity.pendingTransactionsStats(),
        this._api.parity.localTransactions()
      ])
      .then(([pending, stats, local]) => {
        pending
          .filter((transaction) => local[transaction.hash])
          .forEach((transaction) => {
            local[transaction.hash].transaction = transaction;
            local[transaction.hash].stats = stats[transaction.hash].stats;
          });

        const localTransactions = Object
          .keys(local)
          .map((hash) => {
            const data = local[hash];

            data.txHash = hash;
            return data;
          });

        localTransactions.sort((a, b) => {
          a = a.transaction || {};
          b = b.transaction || {};

          if (a.from && b.from && a.from !== b.from) {
            return a.from < b.from;
          }

          if (!a.nonce || !b.nonce) {
            return !a.nonce ? 1 : -1;
          }

          return new BigNumber(a.nonce || 0).cmp(new BigNumber(b.nonce || 0));
        });

        this.setLocalTransactions(localTransactions);
      })
      .then(nextTimeout)
      .catch(nextTimeout);
  }
}
