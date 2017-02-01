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

import sinon from 'sinon';

import AccountStore from './accountStore';

const DEFAULT_ACCOUNT = '0x2345678901';
const ACCOUNTS = {
  '0x1234567890': { uuid: 123 },
  [DEFAULT_ACCOUNT]: { uuid: 234 },
  '0x3456789012': {}
};

let api;
let stubSubscribe;
let store;

function createApi () {
  stubSubscribe = sinon.stub.resolves(1);

  api = {
    subscribe: (params, callback) => {
      callback(null, DEFAULT_ACCOUNT);

      return stubSubscribe(params, callback);
    },
    parity: {
      allAccountsInfo: sinon.stub().resolves(ACCOUNTS),
      getNewDappsWhitelist: sinon.stub().resolves(null)
    }
  };

  return api;
}

function create () {
  store = new AccountStore(createApi());

  return store;
}

describe('views/ParityBar', () => {
  beforeEach(() => {
    create();
  });
});
