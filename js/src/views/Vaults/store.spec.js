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
    describe('setOpenAdd', () => {
      it('sets the isOpenAdd state', () => {
        store.setOpenAdd('testing');
        expect(store.isOpenAdd).to.equal('testing');
      });
    });
  });

  describe('operations', () => {
    describe('closeCreateModal', () => {
      beforeEach(() => {
        store.setModalCreateOpen(true);
        store.closeCreateModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalCreateOpen).to.be.false;
      });
    });

    describe('openCreateModal', () => {
      beforeEach(() => {
        store.openCreateModal();
      });

      it('sets the opened state to true', () => {
        expect(store.isModalCreateOpen).to.be.true;
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
        expect(store.vaultNames.peek()).to.deep.equal(TEST_VAULTS_ALL);
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
