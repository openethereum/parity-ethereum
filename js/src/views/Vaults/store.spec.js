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

import Vaults from './';

import ERRORS from '~/modals/CreateAccount/errors';
import { createApi, TEST_VAULTS_ALL, TEST_VAULTS_META, TEST_VAULTS_OPEN } from './vaults.test.js';

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
    describe('clearCreateFields', () => {
      beforeEach(() => {
        store.setCreateDescription('testing desc');
        store.setCreateName('testing 123');
        store.setCreatePassword('blah');
        store.setCreatePasswordRepeat('bleh');
        store.setCreatePasswordHint('hint');

        store.clearCreateFields();
      });

      it('resets create fields', () => {
        expect(store.createDescription).to.equal('');
        expect(store.createName).to.equal('');
        expect(store.createNameError).not.to.be.null;
        expect(store.createPassword).to.equal('');
        expect(store.createPasswordRepeat).to.equal('');
        expect(store.createPasswordHint).to.equal('');
      });
    });

    describe('setBusyAccounts', () => {
      it('sets the flag', () => {
        store.setBusyAccounts('busy');

        expect(store.isBusyAccounts).to.equal('busy');
      });
    });

    describe('setBusyClose', () => {
      it('sets the flag', () => {
        store.setBusyClose('busy');

        expect(store.isBusyClose).to.equal('busy');
      });
    });

    describe('setBusyCreate', () => {
      it('sets the flag', () => {
        store.setBusyCreate('busy');

        expect(store.isBusyCreate).to.equal('busy');
      });
    });

    describe('setBusyLoad', () => {
      it('sets the flag', () => {
        store.setBusyLoad('busy');

        expect(store.isBusyLoad).to.equal('busy');
      });
    });

    describe('setBusyOpen', () => {
      it('sets the flag', () => {
        store.setBusyOpen('busy');

        expect(store.isBusyOpen).to.equal('busy');
      });
    });

    describe('setCreateDescription', () => {
      it('sets the description', () => {
        store.setCreateDescription('test');

        expect(store.createDescription).to.equal('test');
      });
    });

    describe('setCreateName', () => {
      it('sets the name as passed', () => {
        store.setCreateName('testing');

        expect(store.createName).to.equal('testing');
      });

      it('sets error noName error when empty', () => {
        store.setCreateName(null);

        expect(store.createNameError).to.equal(ERRORS.noName);
      });

      it('sets error duplicateName when duplicated', () => {
        store.setVaults(['testDupe'], [], ['testing']);
        store.setCreateName('testDUPE');

        expect(store.createNameError).to.equal(ERRORS.duplicateName);
      });
    });

    describe('setCreatePassword', () => {
      it('sets the password', () => {
        store.setCreatePassword('testPassword');

        expect(store.createPassword).to.equal('testPassword');
      });
    });

    describe('setCreatePasswordRepeat', () => {
      it('sets the password', () => {
        store.setCreatePasswordRepeat('testPassword');

        expect(store.createPasswordRepeat).to.equal('testPassword');
      });
    });

    describe('setCreatePasswordHint', () => {
      it('sets the password hint', () => {
        store.setCreatePasswordHint('test hint');

        expect(store.createPasswordHint).to.equal('test hint');
      });
    });

    describe('setModalAccountsOpen', () => {
      it('sets the flag', () => {
        store.setModalAccountsOpen('opened');

        expect(store.isModalAccountsOpen).to.equal('opened');
      });
    });

    describe('setModalCloseOpen', () => {
      it('sets the flag', () => {
        store.setModalCloseOpen('opened');

        expect(store.isModalCloseOpen).to.equal('opened');
      });
    });

    describe('setModalCreateOpen', () => {
      it('sets the flag', () => {
        store.setModalCreateOpen('opened');

        expect(store.isModalCreateOpen).to.equal('opened');
      });
    });

    describe('setModalOpenOpen', () => {
      beforeEach(() => {
        store.setVaultPassword('testing');
        store.setModalOpenOpen('opened');
      });

      it('sets the flag', () => {
        expect(store.isModalOpenOpen).to.equal('opened');
      });

      it('rests the password to empty', () => {
        expect(store.vaultPassword).to.equal('');
      });
    });

    describe('setSelectedAccounts', () => {
      it('sets the selected accounts', () => {
        store.setSelectedAccounts('testing');

        expect(store.selectedAccounts).to.equal('testing');
      });
    });

    describe('setVaults', () => {
      beforeEach(() => {
        store.setVaults(['TEST', 'some'], ['TEST'], ['metaTest', 'metaSome']);
      });

      it('stores the available vault names (lookup)', () => {
        expect(store.vaultNames.peek()).to.deep.equal(['test', 'some']);
      });

      it('sets all vaults with correct flags', () => {
        expect(store.vaults.peek()).to.deep.equal([
          { name: 'TEST', meta: 'metaTest', isOpen: true },
          { name: 'some', meta: 'metaSome', isOpen: false }
        ]);
      });
    });

    describe('setVaultName', () => {
      it('sets the vault name', () => {
        store.setVaultName('testName');

        expect(store.vaultName).to.equal('testName');
      });
    });

    describe('setVaultPassword', () => {
      it('sets the vault password', () => {
        store.setVaultPassword('testPassword');

        expect(store.vaultPassword).to.equal('testPassword');
      });
    });

    describe('toggleSelectedAccount', () => {
      beforeEach(() => {
        store.toggleSelectedAccount('123');
      });

      it('adds the selected account', () => {
        expect(store.selectedAccounts['123']).to.be.true;
      });

      it('reverses when toggled again', () => {
        store.toggleSelectedAccount('123');
        expect(store.selectedAccounts['123']).to.be.false;
      });
    });
  });

  describe('@computed', () => {
    describe('createPasswordRepeatError', () => {
      beforeEach(() => {
        store.setCreatePassword('blah');
        store.setCreatePasswordRepeat('bleh');
      });

      it('has error when passwords do not match', () => {
        expect(store.createPasswordRepeatError).not.to.be.null;
      });

      it('has no error when passwords match', () => {
        store.setCreatePasswordRepeat('blah');
        expect(store.createPasswordRepeatError).to.be.null;
      });
    });
  });

  describe('operations', () => {
    describe('closeAccountsModal', () => {
      beforeEach(() => {
        store.setModalAccountsOpen(true);
        store.closeAccountsModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalAccountsOpen).to.be.false;
      });
    });

    describe('closeCloseModal', () => {
      beforeEach(() => {
        store.setModalCloseOpen(true);
        store.closeCloseModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalCloseOpen).to.be.false;
      });
    });

    describe('closeCreateModal', () => {
      beforeEach(() => {
        store.setModalCreateOpen(true);
        store.closeCreateModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalCreateOpen).to.be.false;
      });
    });

    describe('closeOpenModal', () => {
      beforeEach(() => {
        store.setModalOpenOpen(true);
        store.closeOpenModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalOpenOpen).to.be.false;
      });
    });

    describe('openAccountsModal', () => {
      beforeEach(() => {
        store.setSelectedAccounts({ '123': true, '456': false });
        store.openAccountsModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalAccountsOpen).to.be.true;
      });

      it('stores the name', () => {
        expect(store.vaultName).to.equal('testing');
      });

      it('empties the selectedAccounts', () => {
        expect(Object.keys(store.selectedAccounts).length).to.equal(0);
      });
    });

    describe('openCloseModal', () => {
      beforeEach(() => {
        store.openCloseModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalCloseOpen).to.be.true;
      });

      it('stores the name', () => {
        expect(store.vaultName).to.equal('testing');
      });
    });

    describe('openCreateModal', () => {
      beforeEach(() => {
        sinon.spy(store, 'clearCreateFields');
        store.openCreateModal();
      });

      afterEach(() => {
        store.clearCreateFields.restore();
      });

      it('sets the opened state to true', () => {
        expect(store.isModalCreateOpen).to.be.true;
      });

      it('clears the create fields', () => {
        expect(store.clearCreateFields).to.have.been.called;
      });
    });

    describe('openOpenModal', () => {
      beforeEach(() => {
        store.openOpenModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalOpenOpen).to.be.true;
      });

      it('stores the name', () => {
        expect(store.vaultName).to.equal('testing');
      });
    });

    describe('loadVaults', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyLoad');
        sinon.spy(store, 'setVaults');

        return store.loadVaults();
      });

      afterEach(() => {
        store.setBusyLoad.restore();
        store.setVaults.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyLoad).to.have.been.calledWith(true);
        expect(store.isBusyLoad).to.be.false;
      });

      it('calls parity_listVaults', () => {
        expect(api.parity.listVaults).to.have.been.called;
      });

      it('calls parity_listOpenedVaults', () => {
        expect(api.parity.listOpenedVaults).to.have.been.called;
      });

      it('sets the vaults', () => {
        expect(store.setVaults).to.have.been.calledWith(TEST_VAULTS_ALL, TEST_VAULTS_OPEN, [
          TEST_VAULTS_META, TEST_VAULTS_META, TEST_VAULTS_META
        ]);
      });
    });

    describe('closeVault', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyClose');

        store.setVaultName('testVault');

        return store.closeVault();
      });

      afterEach(() => {
        store.setBusyClose.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyClose).to.have.been.calledWith(true);
        expect(store.isBusyClose).to.be.false;
      });

      it('calls into parity_closeVault', () => {
        expect(api.parity.closeVault).to.have.been.calledWith('testVault');
      });
    });

    describe('createVault', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyCreate');

        store.setCreateDescription('testDescription');
        store.setCreateName('testCreateName');
        store.setCreatePassword('testCreatePassword');
        store.setCreatePasswordRepeat('testCreatePassword');
        store.setCreatePasswordHint('testCreateHint');

        return store.createVault();
      });

      afterEach(() => {
        store.setBusyCreate.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyCreate).to.have.been.calledWith(true);
        expect(store.isBusyCreate).to.be.false;
      });

      it('calls into parity_newVault', () => {
        expect(api.parity.newVault).to.have.been.calledWith('testCreateName', 'testCreatePassword');
      });

      it('calls into parity_setVaultMeta', () => {
        expect(api.parity.setVaultMeta).to.have.been.calledWith('testCreateName', {
          description: 'testDescription',
          passwordHint: 'testCreateHint'
        });
      });
    });

    describe('openVault', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyOpen');

        store.setVaultName('testVault');

        return store.openVault();
      });

      afterEach(() => {
        store.setBusyOpen.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyOpen).to.have.been.calledWith(true);
        expect(store.isBusyOpen).to.be.false;
      });

      it('calls into parity_openVault', () => {
        expect(api.parity.openVault).to.have.been.calledWith('testVault');
      });
    });

    describe('moveAccounts', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyAccounts');

        return store.moveAccounts('testVault', ['A', 'B'], ['C']);
      });

      afterEach(() => {
        store.setBusyAccounts.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyAccounts).to.have.been.calledWith(true);
        expect(store.isBusyAccounts).to.be.false;
      });

      it('calls into parity_changeVault', () => {
        expect(api.parity.changeVault).to.have.been.calledWith('A', 'testVault');
        expect(api.parity.changeVault).to.have.been.calledWith('B', 'testVault');
        expect(api.parity.changeVault).to.have.been.calledWith('C', '');
      });
    });
  });
});
