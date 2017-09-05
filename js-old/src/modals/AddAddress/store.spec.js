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

import { TEST_ADDR_A, TEST_ADDR_B, TEST_CONTACTS } from './store.test.js';

describe('modals/AddAddress/store', () => {
  let store;

  describe('@action', () => {
    beforeEach(() => {
      store = new Store(null, TEST_CONTACTS);
    });

    describe('setAddress', () => {
      it('successfully sets non-existent addresses', () => {
        store.setAddress(TEST_ADDR_B);

        expect(store.addressError).to.be.null;
        expect(store.address).to.equal(TEST_ADDR_B);
      });

      it('fails on invalid addresses', () => {
        store.setAddress('0xinvalid');

        expect(store.addressError).not.to.be.null;
      });

      it('fails when an address is already added', () => {
        store.setAddress(TEST_ADDR_A);

        expect(store.addressError).not.to.be.null;
      });
    });

    describe('setName', () => {
      it('sucessfully sets valid names', () => {
        const name = 'Test Name';

        store.setName(name);

        expect(store.nameError).to.be.null;
        expect(store.name).to.equal(name);
      });

      it('fails when name is invalid', () => {
        store.setName(null);

        expect(store.nameError).not.to.be.null;
      });
    });
  });

  describe('@computed', () => {
    beforeEach(() => {
      store = new Store(null, TEST_CONTACTS);
    });

    describe('hasError', () => {
      beforeEach(() => {
        store.setAddress(TEST_ADDR_B);
        store.setName('Test Name');
      });

      it('returns false proper inputs', () => {
        expect(store.hasError).to.be.false;
      });

      it('returns true with addressError', () => {
        store.setAddress(TEST_ADDR_A);

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

  describe('methods', () => {
    let api;

    beforeEach(() => {
      api = {
        parity: {
          setAccountMeta: sinon.stub().resolves(true),
          setAccountName: sinon.stub().resolves(true)
        }
      };
      store = new Store(api, {});
    });

    describe('add', () => {
      it('calls setAccountMeta', () => {
        store.add();

        expect(api.parity.setAccountMeta).to.have.been.called;
      });

      it('calls setAccountName', () => {
        store.add();

        expect(api.parity.setAccountName).to.have.been.called;
      });
    });
  });
});
