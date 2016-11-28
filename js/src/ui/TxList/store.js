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

import { action, observable, transaction } from 'mobx';

export default class Store {
  @observable blocks = {};
  @observable sortedHashes = [];
  @observable transactions = {};

  constructor (api) {
    this._api = api;
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

          return bnB.comparedTo(bnA);
        });
    });
  }

  loadTransactions (_txhashes) {
    const txhashes = _txhashes.filter((txhash) => !this.transactions[txhash]);

    if (!txhashes || !txhashes.length) {
      return;
    }

    Promise
      .all(txhashes.map((txhash) => this._api.eth.getTransactionByHash(txhash)))
      .then((transactions) => {
        this.addTransactions(
          transactions.reduce((transactions, tx, index) => {
            transactions[txhashes[index]] = tx;
            return transactions;
          }, {})
        );

        this.loadBlocks(transactions.map((tx) => tx.blockNumber.toNumber()));
      })
      .catch((error) => {
        console.warn('loadTransactions', error);
      });
  }

  loadBlocks (_blockNumbers) {
    const blockNumbers = Object.keys(
      _blockNumbers.reduce((blockNumbers, bn) => {
        if (!this.blocks[bn]) {
          blockNumbers[bn] = true;
        }

        return blockNumbers;
      }, {})
    );

    if (!blockNumbers.length) {
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
