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

import Store from './store';
import { ACCOUNT, createApi } from './passwordManager.test.js';

let api;
let store;

function createStore (account) {
  api = createApi();
  store = new Store(api, account);

  return store;
}

describe('modals/PasswordManager/Store', () => {
  describe('constructor', () => {
    describe('accounts', () => {
      beforeEach(() => {
        createStore(ACCOUNT);
      });

      it('extracts the address', () => {
        expect(store.address).to.equal(ACCOUNT.address);
      });

      describe('meta', () => {
        it('extracts the full meta', () => {
          expect(toJS(store.meta)).to.deep.equal(ACCOUNT.meta);
        });

        it('extracts the passwordHint', () => {
          expect(store.passwordHint).to.equal(ACCOUNT.meta.passwordHint);
        });
      });
    });
  });
});
