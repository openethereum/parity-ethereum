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

import { mockget as mockEtherscan } from '@parity/etherscan/helpers.spec.js';
import { ADDRESS, createApi } from './transactions.test.js';

import Store from './store';

let api;
let store;

function createStore () {
  api = createApi();
  store = new Store(api);

  return store;
}

function mockQuery () {
  mockEtherscan([{
    query: {
      module: 'account',
      action: 'txlist',
      address: ADDRESS,
      offset: 25,
      page: 1,
      sort: 'desc'
    },
    reply: [{ hash: '123' }]
  }], false, '42');
}

describe('views/Account/Transactions/store', () => {
  beforeEach(() => {
    mockQuery();
    createStore();
  });

  describe('constructor', () => {
    it('sets the api', () => {
      expect(store._api).to.deep.equals(api);
    });

    it('starts with isLoading === false', () => {
      expect(store.isLoading).to.be.false;
    });

    it('starts with isTracing === false', () => {
      expect(store.isTracing).to.be.false;
    });
  });

  describe('@action', () => {
    describe('setHashes', () => {
      it('clears the loading state', () => {
        store.setLoading(true);
        store.setHashes([]);
        expect(store.isLoading).to.be.false;
      });

      it('sets the hashes from the transactions', () => {
        store.setHashes([{ hash: '123' }, { hash: '456' }]);
        expect(store.txHashes.peek()).to.deep.equal(['123', '456']);
      });
    });

    describe('setAddress', () => {
      it('sets the address', () => {
        store.setAddress(ADDRESS);
        expect(store.address).to.equal(ADDRESS);
      });
    });

    describe('setLoading', () => {
      it('sets the isLoading flag', () => {
        store.setLoading(true);
        expect(store.isLoading).to.be.true;
      });
    });

    describe('setNetVersion', () => {
      it('sets the netVersion', () => {
        store.setNetVersion('testing');
        expect(store.netVersion).to.equal('testing');
      });
    });

    describe('setTracing', () => {
      it('sets the isTracing flag', () => {
        store.setTracing(true);
        expect(store.isTracing).to.be.true;
      });
    });

    describe('updateProps', () => {
      it('retrieves transactions once updated', () => {
        sinon.spy(store, 'getTransactions');
        store.updateProps({});

        expect(store.getTransactions).to.have.been.called;
        store.getTransactions.restore();
      });
    });
  });

  describe('operations', () => {
    describe('getTransactions', () => {
      it('retrieves the hashes via etherscan', () => {
        sinon.spy(store, 'fetchEtherscanTransactions');
        store.setAddress(ADDRESS);
        store.setNetVersion('42');
        store.setTracing(false);

        return store.getTransactions().then(() => {
          expect(store.fetchEtherscanTransactions).to.have.been.called;
          expect(store.txHashes.peek()).to.deep.equal(['123']);
          store.fetchEtherscanTransactions.restore();
        });
      });

      it('retrieves the hashes via tracing', () => {
        sinon.spy(store, 'fetchTraceTransactions');
        store.setAddress(ADDRESS);
        store.setNetVersion('42');
        store.setTracing(true);

        return store.getTransactions().then(() => {
          expect(store.fetchTraceTransactions).to.have.been.called;
          expect(store.txHashes.peek()).to.deep.equal(['123', '098']);
          store.fetchTraceTransactions.restore();
        });
      });
    });

    describe('fetchEtherscanTransactions', () => {
      it('retrieves the transactions', () => {
        store.setAddress(ADDRESS);
        store.setNetVersion('42');

        return store.fetchEtherscanTransactions().then((transactions) => {
          expect(transactions).to.deep.equal([{
            blockNumber: new BigNumber(0),
            from: '',
            hash: '123',
            timeStamp: undefined,
            to: '',
            value: undefined
          }]);
        });
      });
    });

    describe('fetchTraceTransactions', () => {
      it('retrieves the transactions', () => {
        store.setAddress(ADDRESS);
        store.setNetVersion('42');

        return store.fetchTraceTransactions().then((transactions) => {
          expect(transactions).to.deep.equal([
            {
              blockNumber: undefined,
              from: undefined,
              hash: '123',
              to: undefined
            },
            {
              blockNumber: undefined,
              from: undefined,
              hash: '098',
              to: undefined
            }
          ]);
        });
      });
    });
  });
});
