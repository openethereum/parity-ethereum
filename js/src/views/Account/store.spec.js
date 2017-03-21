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

import Store from './store';

let store;

function createStore () {
  store = new Store();
}

describe('views/Account/Store', () => {
  beforeEach(() => {
    createStore();
  });

  describe('constructor', () => {
    it('sets all modal visibility to false', () => {
      expect(store.isDeleteVisible).to.be.false;
      expect(store.isEditVisible).to.be.false;
      expect(store.isFaucetVisible).to.be.false;
      expect(store.isFundVisible).to.be.false;
      expect(store.isPasswordVisible).to.be.false;
      expect(store.isTransferVisible).to.be.false;
      expect(store.isVerificationVisible).to.be.false;
    });
  });

  describe('@action', () => {
    describe('toggleDeleteDialog', () => {
      it('toggles the visibility', () => {
        store.toggleDeleteDialog();
        expect(store.isDeleteVisible).to.be.true;
      });
    });

    describe('toggleEditDialog', () => {
      it('toggles the visibility', () => {
        store.toggleEditDialog();
        expect(store.isEditVisible).to.be.true;
      });
    });

    describe('toggleFaucetDialog', () => {
      it('toggles the visibility', () => {
        store.toggleFaucetDialog();
        expect(store.isFaucetVisible).to.be.true;
      });
    });

    describe('toggleFundDialog', () => {
      it('toggles the visibility', () => {
        store.toggleFundDialog();
        expect(store.isFundVisible).to.be.true;
      });
    });

    describe('togglePasswordDialog', () => {
      it('toggles the visibility', () => {
        store.togglePasswordDialog();
        expect(store.isPasswordVisible).to.be.true;
      });
    });

    describe('toggleTransferDialog', () => {
      it('toggles the visibility', () => {
        store.toggleTransferDialog();
        expect(store.isTransferVisible).to.be.true;
      });
    });

    describe('toggleVerificationDialog', () => {
      it('toggles the visibility', () => {
        store.toggleVerificationDialog();
        expect(store.isVerificationVisible).to.be.true;
      });
    });
  });
});
