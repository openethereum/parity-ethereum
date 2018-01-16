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
import sinon from 'sinon';

import Store from './store';

const SUBID = 123;
const BLOCKS = {
  1: { blockhash: '0x1' },
  2: { blockhash: '0x2' }
};
const TRANSACTIONS = {
  '0x123': { blockNumber: new BigNumber(1) },
  '0x234': { blockNumber: new BigNumber(0) },
  '0x345': { blockNumber: new BigNumber(2) },
  '0x456': { blockNumber: new BigNumber(0) }
};

describe('ui/TxList/store', () => {
  let api;
  let store;

  beforeEach(() => {
    api = {
      subscribe: sinon.stub().resolves(SUBID),
      eth: {
        getBlockByNumber: (blockNumber) => {
          return Promise.resolve(BLOCKS[blockNumber]);
        }
      }
    };
    store = new Store(api, null, []);
  });

  describe('create', () => {
    it('has empty storage', () => {
      expect(store.blocks).to.deep.equal({});
      expect(store.sortedHashes.peek()).to.deep.equal([]);
      expect(store.transactions).to.deep.equal({});
    });
  });

  describe('addBlocks', () => {
    beforeEach(() => {
      Object.keys(BLOCKS)
        .forEach((blockNumber) => {
          store.blocks[blockNumber] = BLOCKS[blockNumber];
        });
    });

    it('adds the blocks to the list', () => {
      expect(store.blocks).to.deep.equal(BLOCKS);
    });
  });

  describe('addTransactions', () => {
    beforeEach(() => {
      Object.keys(TRANSACTIONS)
        .forEach((hash) => {
          store.transactions[hash] = TRANSACTIONS[hash];
          store.addHash(hash);
        });
      store.sortHashes();
    });

    it('adds all transactions to the list', () => {
      expect(store.transactions).to.deep.equal(TRANSACTIONS);
    });

    it('sorts transactions based on blockNumber', () => {
      expect(store.sortedHashes.peek()).to.deep.equal(['0x234', '0x456', '0x345', '0x123']);
    });
  });
});
