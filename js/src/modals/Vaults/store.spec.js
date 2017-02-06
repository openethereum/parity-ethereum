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

import Vaults from './';

import { createApi, TEST_VAULTS_ALL, TEST_VAULTS_OPEN } from './vaults.test.js';

let api;
let store;

function create () {
  api = createApi();
  store = new Vaults.Store(api);

  return store;
}

describe('modals/Vaults/Store', () => {
  beforeEach(() => {
    create();
  });

  describe('@action', () => {
    describe('setListAll', () => {
      it('sets the list of all available', () => {
        const LIST = ['testing'];

        store.setListAll(LIST);
        expect(store.listAll.peek()).to.deep.equal(LIST);
      });
    });

    describe('setListOpened', () => {
      it('sets the list of all opened', () => {
        const LIST = ['testing'];

        store.setListOpened(LIST);
        expect(store.listOpened.peek()).to.deep.equal(LIST);
      });
    });

    describe('setOpen', () => {
      it('sets the isOpen state', () => {
        store.setOpen('testing');
        expect(store.isOpen).to.equal('testing');
      });
    });
  });

  describe('operations', () => {
    describe('closeModal', () => {
      beforeEach(() => {
        store.setOpen(true);
        store.closeModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isOpen).to.be.false;
      });
    });

    describe('openModal', () => {
      beforeEach(() => {
        sinon.stub(store, 'loadVaults');
        store.openModal();
      });

      afterEach(() => {
        store.loadVaults.restore();
      });

      it('sets the opened state to true', () => {
        expect(store.isOpen).to.be.true;
      });

      it('calls into loadVaults', () => {
        expect(store.loadVaults).to.have.been.called;
      });
    });

    describe('loadVaults', () => {
      beforeEach(() => {
        return store.loadVaults();
      });

      it('calls parity_listVaults', () => {
        expect(api.parity.listVaults).to.have.been.called;
      });

      it('sets the available vaults', () => {
        expect(store.listAll.peek()).to.deep.equal(TEST_VAULTS_ALL);
      });

      it('calls parity_listOpenedVaults', () => {
        expect(api.parity.listOpenedVaults).to.have.been.called;
      });

      it('sets the opened vaults', () => {
        expect(store.listOpened.peek()).to.deep.equal(TEST_VAULTS_OPEN);
      });
    });
  });
});
