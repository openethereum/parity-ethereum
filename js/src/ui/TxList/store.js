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
import { action, observable, transaction } from 'mobx';

export default class Store {
  @observable blocks = {};
  @observable transactions = [];

  constructor (api) {
    this._api = api;
    this._transactions = {};
  }

  @action addBlocks = (blocks) => {
    this.blocks = Object.assign({}, this.blocks, blocks);
  }

  @action addTransactions = (transactionsArray) => {
    transaction(() => {
      this._transactions = Object.assign(this._transactions, transactionsArray.reduce((txs, tx) => {
        txs[tx.hash] = tx;
        return txs;
      }, {}));

      this.transactions = Object
        .keys(this._transactions)
        .sort((ahash, bhash) => {
          const bnA = this._transactions[ahash].blockNumber || new BigNumber(0);
          const bnB = this._transactions[bhash].blockNumber || new BigNumber(0);

          return bnB.comparedTo(bnA);
        })
        .map((txhash) => this._transactions[txhash]);
    });
  }

  loadTransactions (_hashes) {
    const hashes = _hashes.filter((txhash) => !this._transactions[txhash]);

    if (!hashes || !hashes.length) {
      return;
    }

    Promise
      .all(hashes.map((txhash) => this._api.eth.getTransactionByHash(txhash)))
      .then((transactions) => {
        this.loadBlocks(transactions.map((transaction) => transaction.blockNumber));
        this.addTransactions(transactions);
      })
      .catch((error) => {
        console.warn('loadTransactions', error);
      });
  }

  loadBlocks (_blockNumbers) {
    const blockNumbers = Object
      .keys(
        _blockNumbers
          .filter((blockNumber) => blockNumber)
          .reduce((blockNumbers, blockNumber) => {
            blockNumbers[blockNumber.toNumber()] = true;
            return blockNumbers;
          }, {})
      )
      .filter((bn) => !this.blocks[bn]);

    if (!blockNumbers || !blockNumbers.length) {
      return;
    }

    Promise
      .all(blockNumbers.map((blockNumber) => this._api.eth.getBlockByNumber(blockNumber)))
      .then((_blocks) => {
        this.addBlocks(
          blockNumbers.reduce((blocks, blockNumber, index) => {
            blocks[blockNumber] = _blocks[index];
            return blocks;
          }, {})
        );
      })
      .catch((error) => {
        console.warn('loadBlocks', error);
      });
  }
}
