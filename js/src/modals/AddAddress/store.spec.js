// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { TEST_ADDR_A, TEST_CONTACTS } from './store.test.js';

describe('modals/AddAddress/store', () => {
  let store;

  describe('@action', () => {
    beforeEach(() => {
      store = new Store(null, TEST_CONTACTS);
    });

    describe('setAddress', () => {

    });
  });

  describe('@computed', () => {
    beforeEach(() => {
      store = new Store(null, TEST_CONTACTS);
    });

    describe('hasError', () => {
      beforeEach(() => {
        store.setAddress(TEST_ADDR_A);
        store.setName('Test Name');
      });

      it('returns false proper inputs', () => {
        expect(store.hasError).to.be.false;
      });

      it('returns true with addressError', () => {
        store.setAddress(null);

        expect(store.addressError).not.to.be.null;
        expect(store.hasError).to.be.true;
      });

      it('returns true with nameError', () => {
        store.setName(null);

        expect(store.nameError).not.to.be.null;
        expect(store.hasError).to.be.true;
      });
    });
  });
});
