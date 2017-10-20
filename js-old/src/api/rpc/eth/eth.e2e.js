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

import { createHttpApi } from '../../../../test/e2e/ethapi';
import { isAddress } from '../../../../test/types';

describe('ethapi.eth', () => {
  const ethapi = createHttpApi();
  const address = '0x63cf90d3f0410092fc0fca41846f596223979195';

  let latestBlockNumber;
  let latestBlockHash;

  describe('accounts', () => {
    it('returns the available accounts', () => {
      return ethapi.eth.accounts().then((accounts) => {
        accounts.forEach((account) => {
          expect(isAddress(account)).to.be.true;
        });
      });
    });
  });

  describe('blockNumber', () => {
    it('returns the current blockNumber', () => {
      return ethapi.eth.blockNumber().then((blockNumber) => {
        latestBlockNumber = blockNumber;
        expect(blockNumber.gt(0xabcde)).to.be.true;
      });
    });
  });

  describe('coinbase', () => {
    it('returns the coinbase', () => {
      return ethapi.eth.coinbase().then((coinbase) => {
        expect(isAddress(coinbase)).to.be.true;
      });
    });
  });

  describe('gasPrice', () => {
    it('returns the current gasPrice', () => {
      return ethapi.eth.gasPrice().then((gasPrice) => {
        expect(gasPrice.gt(0)).to.be.true;
      });
    });
  });

  describe('getBalance', () => {
    it('returns the balance for latest block', () => {
      return ethapi.eth.getBalance(address).then((balance) => {
        expect(balance.gt(0)).to.be.true;
      });
    });

    it('returns the balance for a very early block', () => {
      const atBlock = '0x65432';
      const atValue = '18e07120a6e164fee1b';

      return ethapi.eth
        .getBalance(address, atBlock)
        .then((balance) => {
          expect(balance.toString(16)).to.equal(atValue);
        })
        .catch((error) => {
          // Parity doesn't support pruned-before-block balance lookups
          expect(error.message).to.match(/not supported/);
        });
    });

    it('returns the balance for a recent/out-of-pruning-range block', () => {
      return ethapi.eth
        .getBalance(address, latestBlockNumber.minus(1000))
        .then((balance) => {
          expect(balance.gt(0)).to.be.true;
        });
    });
  });

  describe('getBlockByNumber', () => {
    it('returns the latest block', () => {
      return ethapi.eth.getBlockByNumber().then((block) => {
        expect(block).to.be.ok;
      });
    });

    it('returns a block by blockNumber', () => {
      return ethapi.eth.getBlockByNumber(latestBlockNumber).then((block) => {
        latestBlockHash = block.hash;
        expect(block).to.be.ok;
      });
    });

    it('returns a block by blockNumber (full)', () => {
      return ethapi.eth.getBlockByNumber(latestBlockNumber, true).then((block) => {
        expect(block).to.be.ok;
      });
    });
  });

  describe('getBlockByHash', () => {
    it('returns the specified block', () => {
      return ethapi.eth.getBlockByHash(latestBlockHash).then((block) => {
        expect(block).to.be.ok;
        expect(block.hash).to.equal(latestBlockHash);
      });
    });

    it('returns the specified block (full)', () => {
      return ethapi.eth.getBlockByHash(latestBlockHash, true).then((block) => {
        expect(block).to.be.ok;
        expect(block.hash).to.equal(latestBlockHash);
      });
    });
  });

  describe('getBlockTransactionCountByHash', () => {
    it('returns the transactions of the specified hash', () => {
      return ethapi.eth.getBlockTransactionCountByHash(latestBlockHash).then((count) => {
        expect(count).to.be.ok;
        expect(count.gte(0)).to.be.true;
      });
    });
  });

  describe('getBlockTransactionCountByNumber', () => {
    it('returns the transactions of latest', () => {
      return ethapi.eth.getBlockTransactionCountByNumber().then((count) => {
        expect(count).to.be.ok;
        expect(count.gte(0)).to.be.true;
      });
    });

    it('returns the transactions of a specified number', () => {
      return ethapi.eth.getBlockTransactionCountByNumber(latestBlockNumber).then((count) => {
        expect(count).to.be.ok;
        expect(count.gte(0)).to.be.true;
      });
    });
  });

  describe('getTransactionCount', () => {
    it('returns the count for an address', () => {
      return ethapi.eth.getTransactionCount(address).then((count) => {
        expect(count).to.be.ok;
        expect(count.gte(0x1000c2)).to.be.ok;
      });
    });

    it('returns the count for an address at specified blockNumber', () => {
      return ethapi.eth.getTransactionCount(address, latestBlockNumber).then((count) => {
        expect(count).to.be.ok;
        expect(count.gte(0x1000c2)).to.be.ok;
      });
    });
  });
});
