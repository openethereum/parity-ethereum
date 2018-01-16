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
import { ACCOUNT, createApi } from './passwordManager.test.js';

let api;
let store;

function createStore (account) {
  api = createApi();
  store = new Store(api, account);

  return store;
}

describe('modals/PasswordManager/Store', () => {
  beforeEach(() => {
    createStore(ACCOUNT);
  });

  describe('constructor', () => {
    it('extracts the address', () => {
      expect(store.address).to.equal(ACCOUNT.address);
    });

    describe('meta', () => {
      it('extracts the full meta', () => {
        expect(store.meta).to.deep.equal(ACCOUNT.meta);
      });

      it('extracts the passwordHint', () => {
        expect(store.passwordHint).to.equal(ACCOUNT.meta.passwordHint);
      });
    });
  });

  describe('operations', () => {
    const CUR_PASSWORD = 'aPassW0rd';
    const NEW_PASSWORD = 'br@ndNEW';
    const NEW_HINT = 'something new to test';

    describe('changePassword', () => {
      beforeEach(() => {
        store.setPassword(CUR_PASSWORD);
        store.setNewPasswordHint(NEW_HINT);
        store.setNewPassword(NEW_PASSWORD);
        store.setNewPasswordRepeat(NEW_PASSWORD);
      });

      it('calls parity.testPassword with current password', () => {
        return store.changePassword().then(() => {
          expect(api.parity.testPassword).to.have.been.calledWith(ACCOUNT.address, CUR_PASSWORD);
        });
      });

      it('calls parity.setAccountMeta with new hint', () => {
        return store.changePassword().then(() => {
          expect(api.parity.setAccountMeta).to.have.been.calledWith(ACCOUNT.address, Object.assign({}, ACCOUNT.meta, {
            passwordHint: NEW_HINT
          }));
        });
      });

      it('calls parity.changePassword with the new password', () => {
        return store.changePassword().then(() => {
          expect(api.parity.changePassword).to.have.been.calledWith(ACCOUNT.address, CUR_PASSWORD, NEW_PASSWORD);
        });
      });
    });

    describe('testPassword', () => {
      beforeEach(() => {
        store.setValidatePassword(CUR_PASSWORD);
      });

      it('calls parity.testPassword', () => {
        return store.testPassword().then(() => {
          expect(api.parity.testPassword).to.have.been.calledWith(ACCOUNT.address, CUR_PASSWORD);
        });
      });

      it('sets the infoMessage for success/failure', () => {
        return store.testPassword().then(() => {
          expect(store.infoMessage).not.to.be.null;
        });
      });
    });
  });
});
