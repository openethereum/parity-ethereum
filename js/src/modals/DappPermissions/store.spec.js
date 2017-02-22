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
const WHITELIST = ['456', '789'];

let api;
let store;

function create () {
  api = {
    parity: {
      getNewDappsAddresses: sinon.stub().resolves(WHITELIST),
      getNewDappsDefaultAddress: sinon.stub().resolves(WHITELIST[0]),
      setNewDappsAddresses: sinon.stub().resolves(true),
      setNewDappsDefaultAddress: sinon.stub().resolves(true)
    }
  };

  store = new Store(api);
}

describe('modals/DappPermissions/store', () => {
  beforeEach(() => {
    create();
  });

  describe('constructor', () => {
    it('retrieves the whitelist via api', () => {
      expect(api.parity.getNewDappsAddresses).to.be.calledOnce;
    });

    it('sets the retrieved whitelist', () => {
      expect(store.whitelist.peek()).to.deep.equal(WHITELIST);
    });
  });

  describe('@actions', () => {
    beforeEach(() => {
      store.openModal(ACCOUNTS);
    });

    describe('openModal', () => {
      it('sets the modalOpen status', () => {
        expect(store.modalOpen).to.be.true;
      });

      it('sets accounts with checked interfaces', () => {
        expect(store.accounts.peek()).to.deep.equal([
          { address: '123', name: '123', description: '123', default: false, checked: false },
          { address: '456', name: '456', description: '456', default: true, checked: true },
          { address: '789', name: '789', description: '789', default: false, checked: true }
        ]);
      });
    });

    describe('closeModal', () => {
      beforeEach(() => {
        store.setDefaultAccount('789');
        store.closeModal();
      });

      it('calls setNewDappsAddresses', () => {
        expect(api.parity.setNewDappsAddresses).to.have.been.calledWith(['456', '789']);
      });

      it('calls into setNewDappsDefaultAddress', () => {
        expect(api.parity.setNewDappsDefaultAddress).to.have.been.calledWith('789');
      });
    });

    describe('selectAccount', () => {
      beforeEach(() => {
        store.selectAccount('123');
        store.selectAccount('789');
      });

      it('unselects previous selected accounts', () => {
        expect(store.accounts.find((account) => account.address === '123').checked).to.be.true;
      });

      it('selects previous unselected accounts', () => {
        expect(store.accounts.find((account) => account.address === '789').checked).to.be.false;
      });

      it('sets a new default when default was unselected', () => {
        store.selectAccount('456');
        expect(store.accounts.find((account) => account.address === '456').default).to.be.false;
        expect(store.accounts.find((account) => account.address === '123').default).to.be.true;
      });

      it('does not deselect the last account', () => {
        store.selectAccount('123');
        store.selectAccount('456');
        console.log(store.accounts.map((account) => ({ address: account.address, checked: account.checked })));
        expect(store.accounts.find((account) => account.address === '456').default).to.be.true;
        expect(store.accounts.find((account) => account.address === '456').checked).to.be.true;
      });
    });

    describe('setDefaultAccount', () => {
      beforeEach(() => {
        store.setDefaultAccount('789');
      });

      it('unselects previous default', () => {
        expect(store.accounts.find((account) => account.address === '456').default).to.be.false;
      });

      it('selects new default', () => {
        expect(store.accounts.find((account) => account.address === '789').default).to.be.true;
      });
    });
  });
});
