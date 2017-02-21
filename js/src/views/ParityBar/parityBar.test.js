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

const ACCOUNT_DEFAULT = '0x2345678901';
const ACCOUNT_FIRST = '0x1234567890';
const ACCOUNT_NEW = '0x0987654321';
const ACCOUNTS = {
  [ACCOUNT_FIRST]: { uuid: 123 },
  [ACCOUNT_DEFAULT]: { uuid: 234 },
  '0x3456789012': {},
  [ACCOUNT_NEW]: { uuid: 456 }
};

function createApi () {
  const api = {
    subscribe: (params, callback) => {
      callback(null, ACCOUNT_DEFAULT);

      return Promise.resolve(1);
    },
    parity: {
      defaultAccount: sinon.stub().resolves(ACCOUNT_DEFAULT),
      allAccountsInfo: sinon.stub().resolves(ACCOUNTS),
      getNewDappsAddresses: sinon.stub().resolves(null),
      setNewDappsAddresses: sinon.stub().resolves(true),
      setNewDappsDefaultAddress: sinon.stub().resolves(true)
    }
  };

  sinon.spy(api, 'subscribe');

  return api;
}

export {
  ACCOUNT_DEFAULT,
  ACCOUNT_FIRST,
  ACCOUNT_NEW,
  ACCOUNTS,
  createApi
};
