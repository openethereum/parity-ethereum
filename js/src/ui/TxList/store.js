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

import { action, observable, transaction } from 'mobx';
import { uniq } from 'lodash';

export default class Store {
  @observable blocks = {};
  @observable sortedHashes = [];
  @observable transactions = {};

  constructor (api) {
    this._api = api;
    this._subscriptionId = 0;
    this._pendingHashes = [];

    this.subscribe();
  }

  @action addBlocks = (blocks) => {
    this.blocks = Object.assign({}, this.blocks, blocks);
  }

  @action addTransactions = (transactions) => {
    transaction(() => {
      this.transactions = Object.assign({}, this.transactions, transactions);
      this.sortedHashes = Object
        .keys(this.transactions)
        .sort((ahash, bhash) => {
          const bnA = this.transactions[ahash].blockNumber;
          const bnB = this.transactions[bhash].blockNumber;

          if (bnB.eq(0)) {
            return bnB.eq(bnA) ? 0 : 1;
          } else if (bnA.eq(0)) {
            return -1;
          }

          return bnB.comparedTo(bnA);
        });
      this._pendingHashes = this.sortedHashes.filter((hash) => this.transactions[hash].blockNumber.eq(0));
    });
  }

  @action clearPending () {
    this._pendingHashes = [];
  }

  subscribe () {
    this._api
      .subscribe('eth_blockNumber', (error, blockNumber) => {
        if (error) {
          return;
        }

        if (this._pendingHashes.length) {
          this.loadTransactions(this._pendingHashes);
          this.clearPending();
        }
      })
      .then((subscriptionId) => {
        this._subscriptionId = subscriptionId;
      });
  }

  unsubscribe () {
    if (!this._subscriptionId) {
      return;
    }

    this._api.unsubscribe(this._subscriptionId);
    this._subscriptionId = 0;
  }

  loadTransactions (_txhashes) {
    const txhashes = _txhashes.filter((hash) => !this.transactions[hash] || this._pendingHashes.includes(hash));

    if (!txhashes || !txhashes.length) {
      return;
    }

    Promise
      .all(txhashes.map((txhash) => this._api.eth.getTransactionByHash(txhash)))
      .then((_transactions) => {
        const transactions = _transactions.filter((tx) => tx);

        this.addTransactions(
          transactions.reduce((transactions, tx, index) => {
            transactions[txhashes[index]] = tx;
            return transactions;
          }, {})
        );

        this.loadBlocks(transactions.map((tx) => tx.blockNumber ? tx.blockNumber.toNumber() : 0));
      })
      .catch((error) => {
        console.warn('loadTransactions', error);
      });
  }

  loadBlocks (_blockNumbers) {
    const blockNumbers = uniq(_blockNumbers).filter((bn) => !this.blocks[bn]);

    if (!blockNumbers || !blockNumbers.length) {
      return;
    }

    Promise
      .all(blockNumbers.map((blockNumber) => this._api.eth.getBlockByNumber(blockNumber)))
      .then((blocks) => {
        this.addBlocks(
          blocks.reduce((blocks, block, index) => {
            blocks[blockNumbers[index]] = block;
            return blocks;
          }, {})
        );
      })
      .catch((error) => {
        console.warn('loadBlocks', error);
      });
  }
}
