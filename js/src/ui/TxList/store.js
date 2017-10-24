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
import BigNumber from 'bignumber.js';

export default class Store {
  @observable blocks = {};
  @observable sortedHashes = [];
  @observable transactions = {};

  constructor (api, onNewError, hashes) {
    this._api = api;
    this._onNewError = onNewError;
    this.loadTransactions(hashes);
  }

  @action addHash = (hash) => {
    if (!this.sortedHashes.includes(hash)) {
      this.sortedHashes.push(hash);
      this.sortHashes();
    }
  }

  @action removeHash = (hash) => {
    this.sortedHashes.remove(hash);
    let tx = this.transactions[hash];

    if (tx) {
      delete this.transactions[hash];
      delete this.blocks[tx.blockNumber];
    }
    this.sortHashes();
  }

  containsAll = (arr1, arr2) => {
    return arr2.every((arr2Item) => arr1.includes(arr2Item));
  }

  sameHashList = (transactions) => {
    return this.containsAll(transactions, this.sortedHashes) && this.containsAll(this.sortedHashes, transactions);
  }

  sortHashes = () => {
    this.sortedHashes = this.sortedHashes.sort((hashA, hashB) => {
      const bnA = this.transactions[hashA].blockNumber;
      const bnB = this.transactions[hashB].blockNumber;

      // 0 is a special case (has not been added to the blockchain yet)
      if (bnB.eq(0)) {
        return bnB.eq(bnA) ? 0 : 1;
      } else if (bnA.eq(0)) {
        return -1;
      }

      return bnB.comparedTo(bnA);
    });
  }

  loadTransactions (_txhashes) {
    // Ignore special cases and if the contents of _txhashes && this.sortedHashes are the same
    if (Array.isArray(_txhashes) || this.sameHashList(_txhashes)) {
      return;
    }

    // Remove any tx that are edited/cancelled
    this.sortedHashes.forEach((hash) => {
      if (!_txhashes.includes(hash)) {
        this.removeHash(hash);
      }
    });

    // Add any new tx
    _txhashes.forEach((txhash) => {
      if (this.sortedHashes.includes(txhash)) {
        return;
      }

      this._api.eth
        .getTransactionByHash(txhash)
        .then((tx) => {
          if (!tx) {
            return;
          }

          this.transactions[txhash] = tx;

          // If the tx has a blockHash, let's get the blockNumber, otherwise it's ready to be added
          if (tx.blockHash) {
            this._api.parity
              .getBlockHeaderByNumber(tx.blockNumber)
              .then((block) => {
                this.blocks[tx.blockNumber] = block;
                this.addHash(txhash);
              });
          } else {
            this.addHash(txhash);
          }
        });
    });
  }

  cancelTransaction = (txComponent, tx) => {
    const { hash } = tx;

    this._api.parity
      .removeTransaction(hash)
      .then(() => {
        txComponent.setState({ canceled: true });
      })
      .catch((err) => {
        this._onNewError({ message: err });
      });
  }

  killTransaction = (txComponent, tx) => {
    const { hash, gasPrice, from } = tx;

    this._api.parity
      .removeTransaction(hash)
      .then(() => {
        return this._api.parity.postTransaction({
          from: from,
          to: from,                    // set to owner
          gas: new BigNumber(21000),   // set default gas
          gasPrice: gasPrice.times(1.25),   // must be a minimum of 10% growth to be recognized as a replacement by miners (incentive)
          value: new BigNumber(0),     // zero out the value
          condition: null,             // ensure to post this instantly
          data: '0x'
        });
      })
      .then(() => {
        tx.Component.setState({ canceled: true });
      })
      .catch((err) => {
        this._onNewError({ message: err });
      });
  }

  editTransaction = (txComponent, tx) => {
    const { hash, gas, gasPrice, to, from, value, input, condition } = tx;

    this._api.parity
      .removeTransaction(hash)
      .then(() => {
        return this._api.parity.postTransaction({
          from,
          to,
          gas,
          gasPrice,
          value,
          condition,
          data: input
        });
      })
      .then(() => {
        txComponent.setState({ editing: true });
      })
      .catch((err) => {
        this._onNewError({ message: err });
      });
  }
}
