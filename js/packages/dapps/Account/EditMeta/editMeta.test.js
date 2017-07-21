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

const ACCOUNT = {
  address: '0x123456789a123456789a123456789a123456789a',
  meta: {
    description: 'Call me bob',
    passwordHint: 'some hint',
    tags: ['testing']
  },
  name: 'Bobby',
  uuid: '123-456'
};

const ADDRESS = {
  address: '0x0123456789012345678901234567890123456789',
  meta: {
    description: 'Some address',
    extraMeta: {
      some: 'random',
      extra: {
        meta: 'data'
      }
    }
  },
  name: 'Random address'
};

function createApi () {
  return {
    parity: {
      setAccountName: sinon.stub().resolves(),
      setAccountMeta: sinon.stub().resolves(),
      listVaults: sinon.stub().resolves([]),
      listOpenedVaults: sinon.stub().resolves([])
    }
  };
}

function createRedux () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {};
    }
  };
}

export {
  ACCOUNT,
  ADDRESS,
  createApi,
  createRedux
};
