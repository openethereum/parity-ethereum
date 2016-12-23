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

import { toJS } from 'mobx';
import sinon from 'sinon';

import Store from './store';
import { ACCOUNT, ADDRESS } from './editMeta.test.js';

let api;
let store;

function createStore (account) {
  api = {
    parity: {
      setAccountName: sinon.stub().resolves(),
      setAccountMeta: sinon.stub().resolves()
    }
  };

  store = new Store(api, account);
}

describe('modals/EditMeta/Store', () => {
  describe('creation', () => {
    describe('accounts', () => {
      beforeEach(() => {
        createStore(ACCOUNT);
      });

      it('flags it as an account', () => {
        expect(store.isAccount).to.be.true;
      });

      it('extracts the address', () => {
        expect(store.address).to.equal(ACCOUNT.address);
      });

      it('extracts the name', () => {
        expect(store.name).to.equal(ACCOUNT.name);
      });

      it('extracts the tags', () => {
        expect(store.tags.peek()).to.deep.equal(ACCOUNT.meta.tags);
      });

      describe('meta', () => {
        it('extracts the full meta', () => {
          expect(toJS(store.meta)).to.deep.equal(ADDRESS.meta);
        });

        it('extracts the description', () => {
          expect(store.description).to.equal(ADDRESS.meta.description);
        });
      });
    });

    describe('addresses', () => {
      beforeEach(() => {
        createStore(ADDRESS);
      });

      it('flags it as not an account', () => {
        expect(store.isAccount).to.be.false;
      });

      it('extracts the address', () => {
        expect(store.address).to.equal(ADDRESS.address);
      });

      it('extracts the name', () => {
        expect(store.name).to.equal(ADDRESS.name);
      });

      it('extracts the tags (empty)', () => {
        expect(store.tags.peek()).to.deep.equal([]);
      });
    });
  });
});
