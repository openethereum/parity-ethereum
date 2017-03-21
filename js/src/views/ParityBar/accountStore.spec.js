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

import { ACCOUNT_DEFAULT, ACCOUNT_NEW, createApi } from './parityBar.test.js';

let api;
let store;

function create () {
  api = createApi();
  store = new AccountStore(api);

  return store;
}

describe('views/ParityBar/AccountStore', () => {
  beforeEach(() => {
    create();
  });

  describe('constructor', () => {
    it('subscribes to defaultAccount', () => {
      expect(api.subscribe).to.have.been.calledWith('parity_defaultAccount');
    });
  });

  describe('@action', () => {
    describe('setAccounts', () => {
      it('sets the accounts', () => {
        store.setAccounts('testing');
        expect(store.accounts).to.equal('testing');
      });
    });

    describe('setDefaultAccount', () => {
      it('sets the default account', () => {
        store.setDefaultAccount('testing');
        expect(store.defaultAccount).to.equal('testing');
      });
    });

    describe('setLoading', () => {
      it('sets the loading status', () => {
        store.setLoading('testing');
        expect(store.isLoading).to.equal('testing');
      });
    });
  });

  describe('operations', () => {
    describe('loadAccounts', () => {
      beforeEach(() => {
        sinon.spy(store, 'setAccounts');

        return store.loadAccounts();
      });

      afterEach(() => {
        store.setAccounts.restore();
      });

      it('calls into parity_getNewDappsAddresses', () => {
        expect(api.parity.getNewDappsAddresses).to.have.been.called;
      });

      it('calls into parity_allAccountsInfo', () => {
        expect(api.parity.allAccountsInfo).to.have.been.called;
      });

      it('sets the accounts', () => {
        expect(store.setAccounts).to.have.been.called;
      });
    });

    describe('loadDefaultAccount', () => {
      beforeEach(() => {
        return store.loadDefaultAccount();
      });

      it('load and set the default account', () => {
        expect(store.defaultAccount).to.equal(ACCOUNT_DEFAULT);
      });
    });

    describe('makeDefaultAccount', () => {
      beforeEach(() => {
        return store.makeDefaultAccount(ACCOUNT_NEW);
      });

      it('calls into parity_setNewDappsDefaultAddress', () => {
        expect(api.parity.setNewDappsDefaultAddress).to.have.been.calledWith(ACCOUNT_NEW);
      });
    });
  });
});
