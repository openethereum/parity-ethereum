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

import Vaults from './vaults';

import ERRORS from '../dapp-accounts/CreateAccount/errors';
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
    describe('clearVaultFields', () => {
      beforeEach(() => {
        store.setVaultDescription('testing desc');
        store.setVaultName('testing 123');
        store.setVaultPassword('blah');
        store.setVaultPasswordRepeat('bleh');
        store.setVaultPasswordHint('hint');
        store.setVaultPasswordOld('old');
        store.setVaultTags('tags');

        store.clearVaultFields();
      });

      it('resets create fields', () => {
        expect(store.vaultDescription).to.equal('');
        expect(store.vaultName).to.equal('');
        expect(store.vaultNameError).not.to.be.null;
        expect(store.vaultPassword).to.equal('');
        expect(store.vaultPasswordRepeat).to.equal('');
        expect(store.vaultPasswordHint).to.equal('');
        expect(store.vaultPasswordOld).to.equal('');
        expect(store.vaultTags.length).to.equal(0);
      });
    });

    describe('setBusyAccounts', () => {
      it('sets the flag', () => {
        store.setBusyAccounts('busy');

        expect(store.isBusyAccounts).to.equal('busy');
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

    describe('setBusyLock', () => {
      it('sets the flag', () => {
        store.setBusyLock('busy');

        expect(store.isBusyLock).to.equal('busy');
      });
    });

    describe('setBusyMeta', () => {
      it('sets the flag', () => {
        store.setBusyMeta('busy');

        expect(store.isBusyMeta).to.equal('busy');
      });
    });

    describe('setBusyUnlock', () => {
      it('sets the flag', () => {
        store.setBusyUnlock('busy');

        expect(store.isBusyUnlock).to.equal('busy');
      });
    });

    describe('setModalAccountsOpen', () => {
      it('sets the flag', () => {
        store.setModalAccountsOpen('opened');

        expect(store.isModalAccountsOpen).to.equal('opened');
      });
    });

    describe('setModalCreateOpen', () => {
      it('sets the flag', () => {
        store.setModalCreateOpen('opened');

        expect(store.isModalCreateOpen).to.equal('opened');
      });
    });

    describe('setModalLockOpen', () => {
      it('sets the flag', () => {
        store.setModalLockOpen('opened');

        expect(store.isModalLockOpen).to.equal('opened');
      });
    });

    describe('setModalMetaOpen', () => {
      it('sets the flag', () => {
        store.setModalMetaOpen('opened');

        expect(store.isModalMetaOpen).to.equal('opened');
      });
    });

    describe('setModalUnlockOpen', () => {
      beforeEach(() => {
        store.setVaultPassword('testing');
        store.setModalUnlockOpen('opened');
      });

      it('sets the flag', () => {
        expect(store.isModalUnlockOpen).to.equal('opened');
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

      it('sets the opened vaults', () => {
        expect(store.vaultsOpened.peek()).to.deep.equal([
          { name: 'TEST', meta: 'metaTest', isOpen: true }
        ]);
      });
    });

    describe('setVaultDescription', () => {
      it('sets the description', () => {
        store.setVaultDescription('test');

        expect(store.vaultDescription).to.equal('test');
      });
    });

    describe('setVaultName', () => {
      it('sets the name as passed', () => {
        store.setVaultName('testing');

        expect(store.vaultName).to.equal('testing');
      });

      it('sets the vault when found', () => {
        store.setVaults(['testing'], [], ['meta']);
        store.setVaultName('testing');

        expect(store.vault).to.deep.equal({
          isOpen: false,
          meta: 'meta',
          name: 'testing'
        });
      });

      it('clears the vault when not found', () => {
        store.setVaults(['testing'], [], ['meta']);
        store.setVaultName('testing2');

        expect(store.vault).not.to.be.ok;
      });

      it('sets error noName error when empty', () => {
        store.setVaultName(null);

        expect(store.vaultNameError).to.equal(ERRORS.noName);
      });

      it('sets error duplicateName when duplicated', () => {
        store.setVaults(['testDupe'], [], ['testing']);
        store.setVaultName('testDUPE');

        expect(store.vaultNameError).to.equal(ERRORS.duplicateName);
      });
    });

    describe('setVaultPassword', () => {
      it('sets the password', () => {
        store.setVaultPassword('testPassword');

        expect(store.vaultPassword).to.equal('testPassword');
      });
    });

    describe('setVaultPasswordRepeat', () => {
      it('sets the password', () => {
        store.setVaultPasswordRepeat('testPassword');

        expect(store.vaultPasswordRepeat).to.equal('testPassword');
      });
    });

    describe('setVaultPasswordHint', () => {
      it('sets the password hint', () => {
        store.setVaultPasswordHint('test hint');

        expect(store.vaultPasswordHint).to.equal('test hint');
      });
    });

    describe('setVaultTags', () => {
      it('sets the tags', () => {
        store.setVaultTags('test');

        expect(store.vaultTags).to.equal('test');
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
        store.setVaultPassword('blah');
        store.setVaultPasswordRepeat('bleh');
      });

      it('has error when passwords do not match', () => {
        expect(store.vaultPasswordRepeatError).not.to.be.null;
      });

      it('has no error when passwords match', () => {
        store.setVaultPasswordRepeat('blah');
        expect(store.vaultPasswordRepeatError).to.be.null;
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

    describe('closeCreateModal', () => {
      beforeEach(() => {
        store.setModalCreateOpen(true);
        store.closeCreateModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalCreateOpen).to.be.false;
      });
    });

    describe('closeLockModal', () => {
      beforeEach(() => {
        store.setModalLockOpen(true);
        store.closeLockModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalLockOpen).to.be.false;
      });
    });

    describe('closeMetaModal', () => {
      beforeEach(() => {
        store.setModalMetaOpen(true);
        store.closeMetaModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalMetaOpen).to.be.false;
      });
    });

    describe('closeUnlockModal', () => {
      beforeEach(() => {
        store.setModalUnlockOpen(true);
        store.closeUnlockModal();
      });

      it('sets the opened state to false', () => {
        expect(store.isModalUnlockOpen).to.be.false;
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

    describe('openCreateModal', () => {
      beforeEach(() => {
        sinon.spy(store, 'clearVaultFields');
        store.openCreateModal();
      });

      afterEach(() => {
        store.clearVaultFields.restore();
      });

      it('sets the opened state to true', () => {
        expect(store.isModalCreateOpen).to.be.true;
      });

      it('clears the create fields', () => {
        expect(store.clearVaultFields).to.have.been.called;
      });
    });

    describe('openLockModal', () => {
      beforeEach(() => {
        store.openLockModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalLockOpen).to.be.true;
      });

      it('stores the name', () => {
        expect(store.vaultName).to.equal('testing');
      });
    });

    describe('openMetaModal', () => {
      beforeEach(() => {
        store.openMetaModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalMetaOpen).to.be.true;
      });

      it('stores the name', () => {
        expect(store.vaultName).to.equal('testing');
      });
    });

    describe('openUnlockModal', () => {
      beforeEach(() => {
        store.openUnlockModal('testing');
      });

      it('sets the opened state to true', () => {
        expect(store.isModalUnlockOpen).to.be.true;
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
        sinon.spy(store, 'setBusyLock');

        store.setVaultName('testVault');

        return store.closeVault();
      });

      afterEach(() => {
        store.setBusyLock.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyLock).to.have.been.calledWith(true);
        expect(store.isBusyLock).to.be.false;
      });

      it('calls into parity_closeVault', () => {
        expect(api.parity.closeVault).to.have.been.calledWith('testVault');
      });
    });

    describe('createVault', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyCreate');

        store.setVaultDescription('testDescription');
        store.setVaultName('testCreateName');
        store.setVaultPassword('testCreatePassword');
        store.setVaultPasswordRepeat('testCreatePassword');
        store.setVaultPasswordHint('testCreateHint');
        store.setVaultTags('testTags');

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
          passwordHint: 'testCreateHint',
          tags: 'testTags'
        });
      });
    });

    describe('editVaultMeta', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyMeta');

        store.setVaultDescription('testDescription');
        store.setVaultName('testCreateName');
        store.setVaultPasswordHint('testCreateHint');
        store.setVaultTags('testTags');

        return store.editVaultMeta();
      });

      afterEach(() => {
        store.setBusyMeta.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyMeta).to.have.been.calledWith(true);
        expect(store.isBusyMeta).to.be.false;
      });

      it('calls into parity_setVaultMeta', () => {
        expect(api.parity.setVaultMeta).to.have.been.calledWith('testCreateName', {
          description: 'testDescription',
          passwordHint: 'testCreateHint',
          tags: 'testTags'
        });
      });
    });

    describe('editVaultMeta', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyMeta');

        store.setVaultDescription('testDescription');
        store.setVaultName('testCreateName');
        store.setVaultPasswordHint('testCreateHint');
        store.setVaultTags('testTags');

        return store.editVaultMeta();
      });

      afterEach(() => {
        store.setBusyMeta.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyMeta).to.have.been.calledWith(true);
        expect(store.isBusyMeta).to.be.false;
      });

      it('calls into parity_setVaultMeta', () => {
        expect(api.parity.setVaultMeta).to.have.been.calledWith('testCreateName', {
          description: 'testDescription',
          passwordHint: 'testCreateHint',
          tags: 'testTags'
        });
      });
    });

    describe('editVaultPassword', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyMeta');

        store.setVaultName('testName');
        store.setVaultPasswordOld('oldPassword');
        store.setVaultPassword('newPassword');

        return store.editVaultPassword();
      });

      afterEach(() => {
        store.setBusyMeta.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyMeta).to.have.been.calledWith(true);
        expect(store.isBusyMeta).to.be.false;
      });

      it('calls into parity_openVault', () => {
        expect(api.parity.openVault).to.have.been.calledWith('testName', 'oldPassword');
      });

      it('calls into parity_changeVaultPassword', () => {
        expect(api.parity.changeVaultPassword).to.have.been.calledWith('testName', 'newPassword');
      });
    });

    describe('openVault', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyUnlock');

        store.setVaultName('testVault');

        return store.openVault();
      });

      afterEach(() => {
        store.setBusyUnlock.restore();
      });

      it('sets and resets the busy flag', () => {
        expect(store.setBusyUnlock).to.have.been.calledWith(true);
        expect(store.isBusyUnlock).to.be.false;
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

      it('sets the busy flag', () => {
        expect(store.setBusyAccounts).to.have.been.calledWith(true);
      });

      it('calls into parity_changeVault', () => {
        expect(api.parity.changeVault).to.have.been.calledWith('A', 'testVault');
        expect(api.parity.changeVault).to.have.been.calledWith('B', 'testVault');
        expect(api.parity.changeVault).to.have.been.calledWith('C', '');
      });
    });

    describe('moveAccount', () => {
      beforeEach(() => {
        sinon.spy(store, 'setBusyAccounts');

        return store.moveAccount('testVault', 'A');
      });

      afterEach(() => {
        store.setBusyAccounts.restore();
      });

      it('sets the busy flag', () => {
        expect(store.setBusyAccounts).to.have.been.calledWith(true);
      });

      it('calls into parity_changeVault', () => {
        expect(api.parity.changeVault).to.have.been.calledWith('A', 'testVault');
      });
    });
  });
});
