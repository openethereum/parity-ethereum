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
import { ACCOUNT, ADDRESS, createApi } from './editMeta.test.js';

let api;
let store;
let vaultStore;

function createVaultStore () {
  return {
    moveAccount: sinon.stub().resolves(true)
  };
}

function createStore (account) {
  api = createApi();
  vaultStore = createVaultStore();

  store = new Store(api, account);

  return store;
}

describe('modals/EditMeta/Store', () => {
  describe('constructor', () => {
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
        expect(store.tags).to.deep.equal(ACCOUNT.meta.tags);
      });

      describe('meta', () => {
        it('extracts the full meta', () => {
          expect(store.meta).to.deep.equal(ACCOUNT.meta);
        });

        it('extracts the description', () => {
          expect(store.description).to.equal(ACCOUNT.meta.description);
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

  describe('@computed', () => {
    beforeEach(() => {
      createStore(ADDRESS);
    });

    describe('hasError', () => {
      it('is false when no nameError', () => {
        store.setNameError(null);
        expect(store.hasError).to.be.false;
      });

      it('is false with a nameError', () => {
        store.setNameError('some error');
        expect(store.hasError).to.be.true;
      });
    });
  });

  describe('@actions', () => {
    beforeEach(() => {
      createStore(ADDRESS);
    });

    describe('setBusy', () => {
      it('sets the isBusy flag', () => {
        store.setBusy('testing');
        expect(store.isBusy).to.equal('testing');
      });
    });

    describe('setDescription', () => {
      it('sets the description', () => {
        store.setDescription('description');
        expect(store.description).to.equal('description');
      });
    });

    describe('setName', () => {
      it('sets the name', () => {
        store.setName('valid name');
        expect(store.name).to.equal('valid name');
        expect(store.nameError).to.be.null;
      });

      it('sets name and error on invalid', () => {
        store.setName('');
        expect(store.name).to.equal('');
        expect(store.nameError).not.to.be.null;
      });
    });

    describe('setPasswordHint', () => {
      it('sets the description', () => {
        store.setPasswordHint('passwordHint');
        expect(store.passwordHint).to.equal('passwordHint');
      });
    });

    describe('setTags', () => {
      it('sets the tags', () => {
        store.setTags(['taga', 'tagb']);
        expect(store.tags.peek()).to.deep.equal(['taga', 'tagb']);
      });
    });

    describe('setVaultName', () => {
      it('sets the name', () => {
        store.setVaultName('testing');
        expect(store.vaultName).to.equal('testing');
      });
    });
  });

  describe('operations', () => {
    describe('save', () => {
      beforeEach(() => {
        createStore(ACCOUNT);
        sinon.spy(store, 'setBusy');
      });

      afterEach(() => {
        store.setBusy.restore();
      });

      it('sets the busy flag, clearing it when done', () => {
        return store.save().then(() => {
          expect(store.setBusy).to.have.been.calledWith(true);
          expect(store.setBusy).to.have.been.calledWith(false);
        });
      });

      it('calls parity.setAccountName with the set value', () => {
        store.setName('test name');

        return store.save().then(() => {
          expect(api.parity.setAccountName).to.be.calledWith(ACCOUNT.address, 'test name');
        });
      });

      it('calls parity.setAccountMeta with the adjusted values', () => {
        store.setDescription('some new description');
        store.setPasswordHint('some new passwordhint');
        store.setTags(['taga']);

        return store.save().then(() => {
          expect(api.parity.setAccountMeta).to.have.been.calledWith(
            ACCOUNT.address, Object.assign({}, ACCOUNT.meta, {
              description: 'some new description',
              passwordHint: 'some new passwordhint',
              tags: ['taga']
            })
          );
        });
      });

      it('moves vault account when applicable', () => {
        store.setVaultName('testing');

        return store.save(vaultStore).then(() => {
          expect(vaultStore.moveAccount).to.have.been.calledWith('testing', ACCOUNT.address);
        });
      });

      it('calls parity.setAccountMeta with the adjusted values', () => {
        store.setDescription('some new description');
        store.setPasswordHint('some new passwordhint');
        store.setTags(['taga']);
        store.save();

        expect(api.parity.setAccountMeta).to.have.been.calledWith(ACCOUNT.address, Object.assign({}, ACCOUNT.meta, {
          description: 'some new description',
          passwordHint: 'some new passwordhint',
          tags: ['taga']
        }));
      });
    });
  });
});
