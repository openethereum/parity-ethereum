// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

const ACCOUNTS = { '0x00000123456789abcdef123456789abcdef123456789abcdef': {} };
const GETH_ADDRESSES = ['0x123456789abcdef123456789abcdef123456789abcdef00000'];

function createApi () {
  return {
    eth: {
      getBalance: sinon.stub().resolves(new BigNumber(1))
    },
    parity: {
      generateSecretPhrase: sinon.stub().resolves(),
      listGethAccounts: sinon.stub().resolves(GETH_ADDRESSES),
      phraseToAddress: sinon.stub().resolves()
    }
  };
}

function createStore () {
  return new Store(createApi(), ACCOUNTS);
}

export {
  ACCOUNTS,
  GETH_ADDRESSES,
  createApi,
  createStore
};
