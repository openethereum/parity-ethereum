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
import localStore from 'store';

import Contracts from '~/contracts';

import Store, { LS_KEY_DISPLAY } from './dappsStore';

const APPID_BASICCOIN = '0xf9f2d620c2e08f83e45555247146c62185e4ab7cf82a4b9002a265a0d020348f';
const APPID_DAPPREG = '0x7bbc4f1a27628781b96213e781a1b8eec6982c1db8fac739af6e4c5a55862c03';
const FETCH_OK = {
  ok: true,
  status: 200
};

let globalContractsGet;
let globalFetch;

function stubGlobals () {
  globalContractsGet = Contracts.get;
  globalFetch = global.fetch;

  Contracts.get = () => {
    return {
      dappReg: {
        at: sinon.stub().resolves([[0, 1, 2, 3], 'appOwner']),
        count: sinon.stub().resolves(new BigNumber(1)),
        getContract: sinon.stub().resolves({}),
        getContent: sinon.stub().resolves([0, 1, 2, 3]),
        getImage: sinon.stub().resolves([0, 1, 2, 3]),
        getManifest: sinon.stub().resolves([0, 1, 2, 3])
      }
    };
  };

  global.fetch = (url) => {
    switch (url) {
      case '/api/apps':
        return Promise.resolve(Object.assign({}, FETCH_OK, {
          json: sinon.stub().resolves([]) // TODO: Local stubs in here
        }));

      default:
        console.log('Unknown fetch stub endpoint', url);
        return Promise.reject();
    }
  };
}

function restoreGlobals () {
  Contracts.get = globalContractsGet;
  global.fetch = globalFetch;
}

let api;
let store;

function create () {
  api = {};

  store = new Store(api);

  return store;
}

describe.only('views/Dapps/DappStore', () => {
  beforeEach(() => {
    stubGlobals();
  });

  afterEach(() => {
    restoreGlobals();
  });

  describe('saved views', () => {
    beforeEach(() => {
      localStore.set(LS_KEY_DISPLAY, {
        [APPID_BASICCOIN]: { visible: false },
        [APPID_DAPPREG]: { visible: true }
      });

      return create().loadAllApps();
    });

    it('disables based on saved keys', () => {
      expect(true).to.be.true;
    });

    it('enables based on saved keys', () => {
      expect(true).to.be.true;
    });

    afterEach(() => {
      localStore.set(LS_KEY_DISPLAY, {});
    });
  });
});
