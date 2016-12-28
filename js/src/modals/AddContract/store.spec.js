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

import Store from './store';

import { CONTRACTS, createApi } from './addContract.test.js';

let api;
let store;

function createStore () {
  api = createApi();
  store = new Store(api, CONTRACTS);
}

describe('modals/AddContract/Store', () => {
  beforeEach(() => {
    createStore();
  });

  describe('constructor', () => {
    it('creates an instance', () => {
      expect(store).to.be.ok;
    });
  });

  describe('@actions', () => {
    describe('nextStep/prevStep', () => {
      it('moves to the next/prev step', () => {
        expect(store.step).to.equal(0);
        store.nextStep();
        expect(store.step).to.equal(1);
        store.prevStep();
        expect(store.step).to.equal(0);
      });
    });

    describe('setAddress', () => {
      const INVALID_ADDR = '0x123';
      const VALID_ADDR = '0x5A5eFF38DA95b0D58b6C616f2699168B480953C9';
      const DUPE_ADDR = Object.keys(CONTRACTS)[0];

      it('sets a valid address', () => {
        store.setAddress(VALID_ADDR);
        expect(store.address).to.equal(VALID_ADDR);
        expect(store.addressError).to.be.null;
      });

      it('sets the error on invalid address', () => {
        store.setAddress(INVALID_ADDR);
        expect(store.address).to.equal(INVALID_ADDR);
        expect(store.addressError).not.to.be.null;
      });

      it('sets the error on suplicate address', () => {
        store.setAddress(DUPE_ADDR);
        expect(store.address).to.equal(DUPE_ADDR);
        expect(store.addressError).not.to.be.null;
      });
    });

    describe('setDescription', () => {
      it('sets the description', () => {
        store.setDescription('test description');
        expect(store.description).to.equal('test description');
      });
    });

    describe('setName', () => {
      it('sets the name', () => {
        store.setName('some name');
        expect(store.name).to.equal('some name');
        expect(store.nameError).to.be.null;
      });

      it('sets the error', () => {
        store.setName('s');
        expect(store.name).to.equal('s');
        expect(store.nameError).not.to.be.null;
      });
    });
  });
});
