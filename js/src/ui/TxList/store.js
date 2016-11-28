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
    this._transactions = {};
  }

  @action addBlocks = (blocks) => {
    this.blocks = Object.assign({}, this.blocks, blocks);
  }

  @action addTransaction = (tx) => {
    transaction(() => {
      this.transactions[tx.hash] = tx;
      this.sortedHashes = Object
        .keys(this.transactions)
        .sort((ahash, bhash) => {
          const bnA = this.transactions[ahash].blockNumber;
          const bnB = this.transactions[bhash].blockNumber;

          return bnB.comparedTo(bnA);
        });
    });
  }

  loadTransactions (_hashes) {
    _hashes.forEach((txhash) => {
      if (this._transactions[txhash]) {
        return;
      }

      this._api.eth
        .getTransactionByHash(txhash)
        .then((transaction) => {
          const blockNumber = transaction.blockNumber.toNumber();
          this.addTransaction(transaction);

          if (this.blocks[blockNumber]) {
            return;
          }

          return this._api.eth
            .getBlockByNumber(blockNumber)
            .then((block) => {
              this.addBlocks({ [blockNumber]: block });
            });
        })
        .catch((error) => {
          console.warn('loadTransaction', txhash, error);
        });
    });
  }
}
