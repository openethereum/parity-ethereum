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

import Api from '~/api';
import Store from './store';

const ADDRESS = '0x00000123456789abcdef123456789abcdef123456789abcdef';
const ACCOUNTS = { [ADDRESS]: {} };
const GETH_ADDRESSES = [
  '0x123456789abcdef123456789abcdef123456789abcdef00000',
  '0x00000123456789abcdef123456789abcdef123456789abcdef'
];

let counter = 1;

function createApi () {
  return {
    eth: {
      getBalance: sinon.stub().resolves(new BigNumber(1))
    },
    parity: {
      generateSecretPhrase: sinon.stub().resolves('some account phrase'),
      importGethAccounts: sinon.stub().resolves(GETH_ADDRESSES),
      listGethAccounts: sinon.stub().resolves(GETH_ADDRESSES),
      newAccountFromPhrase: sinon.stub().resolves(ADDRESS),
      newAccountFromSecret: sinon.stub().resolves(ADDRESS),
      newAccountFromWallet: sinon.stub().resolves(ADDRESS),
      phraseToAddress: () => Promise.resolve(`${++counter}`),
      setAccountMeta: sinon.stub().resolves(),
      setAccountName: sinon.stub().resolves(),
      listVaults: sinon.stub().resolves([]),
      listOpenedVaults: sinon.stub().resolves([])
    },
    util: Api.util
  };
}

function createRedux () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        nodeStatus: {
          isTest: true
        }
      };
    }
  };
}

function createStore () {
  return new Store(createApi(), ACCOUNTS);
}

export {
  ACCOUNTS,
  ADDRESS,
  GETH_ADDRESSES,
  createApi,
  createRedux,
  createStore
};
