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

import Store from './store';

const ACCOUNTS = {
  '123': { address: '123', name: '123', meta: { description: '123' } },
  '456': { address: '456', name: '456', meta: { description: '456' } },
  '789': { address: '789', name: '789', meta: { description: '789' } }
};
const WHITELIST = ['123', '456'];

describe('modals/DappPermissions/store', () => {
  let api;
  let store;

  beforeEach(() => {
    api = {
      parity: {
        getNewDappsWhitelist: sinon.stub().resolves(WHITELIST),
        setNewDappsWhitelist: sinon.stub().resolves(true)
      }
    };

    store = new Store(api);
  });

  describe('constructor', () => {
    it('retrieves the whitelist via api', () => {
      expect(api.parity.getNewDappsWhitelist).to.be.calledOnce;
    });

    it('sets the retrieved whitelist', () => {
      expect(store.whitelist.peek()).to.deep.equal(WHITELIST);
    });
  });

  describe('@actions', () => {
    describe('openModal', () => {
      beforeEach(() => {
        store.openModal(ACCOUNTS);
      });

      it('sets the modalOpen status', () => {
        expect(store.modalOpen).to.be.true;
      });

      it('sets accounts with checked interfaces', () => {
        expect(store.accounts.peek()).to.deep.equal([
          { address: '123', name: '123', description: '123', checked: true },
          { address: '456', name: '456', description: '456', checked: true },
          { address: '789', name: '789', description: '789', checked: false }
        ]);
      });
    });

    describe('closeModal', () => {
      beforeEach(() => {
        store.openModal(ACCOUNTS);
        store.selectAccount('789');
        store.closeModal();
      });

      it('calls setNewDappsWhitelist', () => {
        expect(api.parity.setNewDappsWhitelist).to.have.been.calledOnce;
      });
    });

    describe('selectAccount', () => {
      beforeEach(() => {
        store.openModal(ACCOUNTS);
        store.selectAccount('123');
        store.selectAccount('789');
      });

      it('unselects previous selected accounts', () => {
        expect(store.accounts.find((account) => account.address === '123').checked).to.be.false;
      });

      it('selects previous unselected accounts', () => {
        expect(store.accounts.find((account) => account.address === '789').checked).to.be.true;
      });
    });
  });
});
