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

import { createApi } from './createAccount.test.js';

let api;
let store;

function createStore () {
  api = createApi();
  store = new Store(api);

  return store;
}

describe('modals/CreateAccount/Store', () => {
  beforeEach(() => {
    createStore();
  });

  describe('constructor', () => {
    it('sets the initial createType to fromNew', () => {
      expect(store.createType).to.equal('fromNew');
    });

    it('sets the initial stage to create', () => {
      expect(store.stage).to.equal(0);
    });
  });

  describe('@action', () => {
    describe('setAddress', () => {
      const ADDR = '0x1234567890123456789012345678901234567890';

      it('sets the address', () => {
        store.setAddress(ADDR);
        expect(store.address).to.equal(ADDR);
      });
    });

    describe('setCreateType', () => {
      it('allows changing the type', () => {
        store.setCreateType('testing');
        expect(store.createType).to.equal('testing');
      });
    });

    describe('setDescription', () => {
      it('allows setting the description', () => {
        store.setDescription('testing');
        expect(store.description).to.equal('testing');
      });
    });

    describe('setName', () => {
      it('allows setting the name', () => {
        store.setName('testing');
        expect(store.name).to.equal('testing');
      });
    });

    describe('setPhrase', () => {
      it('allows setting the phrase', () => {
        store.setPhrase('testing');
        expect(store.phrase).to.equal('testing');
      });
    });

    describe('setStage', () => {
      it('changes to the provided stage', () => {
        store.setStage(2);
        expect(store.stage).to.equal(2);
      });
    });

    describe('nextStage/prevStage', () => {
      it('changes to next/prev', () => {
        expect(store.stage).to.equal(0);
        store.nextStage();
        expect(store.stage).to.equal(1);
        store.prevStage();
        expect(store.stage).to.equal(0);
      });
    });
  });
});
