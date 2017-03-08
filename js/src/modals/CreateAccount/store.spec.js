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

import { ACCOUNTS, ADDRESS, GETH_ADDRESSES, createApi } from './createAccount.test.js';

let api;
let store;
let vaultStore;

function createVaultStore () {
  vaultStore = {
    moveAccount: sinon.stub().resolves(),
    listVaults: sinon.stub().resolves()
  };

  return vaultStore;
}

function createStore (loadGeth) {
  createVaultStore();

  api = createApi();
  store = new Store(api, ACCOUNTS, loadGeth);

  return store;
}

describe('modals/CreateAccount/Store', () => {
  beforeEach(() => {
    createStore();
  });

  describe('constructor', () => {
    it('captures the accounts passed', () => {
      expect(store.accounts).to.deep.equal(ACCOUNTS);
    });

    it('starts as non-busy', () => {
      expect(store.isBusy).to.be.false;
    });

    it('sets the initial createType to fromNew', () => {
      expect(store.createType).to.equal('fromNew');
    });

    it('sets the initial stage to create', () => {
      expect(store.stage).to.equal(0);
    });

    it('loads the geth accounts', () => {
      expect(store.gethAccountsAvailable.map((account) => account.address)).to.deep.equal([GETH_ADDRESSES[0]]);
    });

    it('does not load geth accounts when loadGeth === false', () => {
      createStore(false);
      expect(store.gethAccountsAvailable.peek()).to.deep.equal([]);
    });
  });

  describe('@action', () => {
    describe('clearErrors', () => {
      beforeEach(() => {
        store.setName('testing');
        store.setPassword('testing');
        store.setVaultName('testing');
        store.setRawKey('test');
        store.setWalletFile('test');
        store.setWalletJson('test');
      });

      it('clears all errors', () => {
        store.clearErrors();

        expect(store.name).to.equal('');
        expect(store.nameError).to.be.null;
        expect(store.password).to.equal('');
        expect(store.passwordRepeatError).to.be.null;
        expect(store.rawKey).to.equal('');
        expect(store.rawKeyError).to.be.null;
        expect(store.vaultName).to.equal('');
        expect(store.walletFile).to.equal('');
        expect(store.walletFileError).to.be.null;
        expect(store.walletJson).to.equal('');
      });
    });

    describe('selectGethAccount', () => {
      it('selects and deselects and address', () => {
        expect(store.gethAddresses.peek()).to.deep.equal([]);
        store.selectGethAccount(GETH_ADDRESSES[0]);
        expect(store.gethAddresses.peek()).to.deep.equal([GETH_ADDRESSES[0]]);
        store.selectGethAccount(GETH_ADDRESSES[0]);
        expect(store.gethAddresses.peek()).to.deep.equal([]);
      });
    });

    describe('setAddress', () => {
      const ADDR = '0x1234567890123456789012345678901234567890';

      it('sets the address', () => {
        store.setAddress(ADDR);
        expect(store.address).to.equal(ADDR);
      });
    });

    describe('setBusy', () => {
      it('sets the busy flag', () => {
        store.setBusy(true);
        expect(store.isBusy).to.be.true;
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
        expect(store.nameError).to.be.null;
      });

      it('sets errors on invalid names', () => {
        store.setName('');
        expect(store.nameError).not.to.be.null;
      });
    });

    describe('setPassword', () => {
      it('allows setting the password', () => {
        store.setPassword('testing');
        expect(store.password).to.equal('testing');
      });
    });

    describe('setPasswordHint', () => {
      it('allows setting the passwordHint', () => {
        store.setPasswordHint('testing');
        expect(store.passwordHint).to.equal('testing');
      });
    });

    describe('setPasswordRepeat', () => {
      it('allows setting the passwordRepeat', () => {
        store.setPasswordRepeat('testing');
        expect(store.passwordRepeat).to.equal('testing');
      });
    });

    describe('setPhrase', () => {
      it('allows setting the phrase', () => {
        store.setPhrase('testing');
        expect(store.phrase).to.equal('testing');
      });
    });

    describe('setRawKey', () => {
      it('sets error when empty key', () => {
        store.setRawKey(null);
        expect(store.rawKeyError).not.to.be.null;
      });

      it('sets error when non-hex value', () => {
        store.setRawKey('0000000000000000000000000000000000000000000000000000000000000000');
        expect(store.rawKeyError).not.to.be.null;
      });

      it('sets error when non-valid length value', () => {
        store.setRawKey('0x0');
        expect(store.rawKeyError).not.to.be.null;
      });

      it('sets the key when checks pass', () => {
        const KEY = '0x1000000000000000000000000000000000000000000000000000000000000000';

        store.setRawKey(KEY);
        expect(store.rawKey).to.equal(KEY);
        expect(store.rawKeyError).to.be.null;
      });
    });

    describe('setStage', () => {
      it('changes to the provided stage', () => {
        store.setStage(2);
        expect(store.stage).to.equal(2);
      });
    });

    describe('setVaultName', () => {
      it('sets the vault name', () => {
        store.setVaultName('testVault');
        expect(store.vaultName).to.equal('testVault');
      });
    });

    describe('setWalletFile', () => {
      it('sets the filepath', () => {
        store.setWalletFile('testing');
        expect(store.walletFile).to.equal('testing');
      });

      it('cleans up the fakepath', () => {
        store.setWalletFile('C:\\fakepath\\testing');
        expect(store.walletFile).to.equal('testing');
      });

      it('sets the error', () => {
        store.setWalletFile('testing');
        expect(store.walletFileError).not.to.be.null;
      });
    });

    describe('setWalletJson', () => {
      it('sets the json', () => {
        store.setWalletJson('testing');
        expect(store.walletJson).to.equal('testing');
      });

      it('clears previous file errors', () => {
        store.setWalletFile('testing');
        store.setWalletJson('testing');
        expect(store.walletFileError).to.be.null;
      });
    });

    describe('setWindowsPhrase', () => {
      it('allows setting the windows toggle', () => {
        store.setWindowsPhrase(true);
        expect(store.isWindowsPhrase).to.be.true;
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

  describe('@computed', () => {
    describe('canCreate', () => {
      beforeEach(() => {
        store.clearErrors();
      });

      describe('createType === fromGeth', () => {
        beforeEach(() => {
          store.setCreateType('fromGeth');
        });

        it('returns false on none selected', () => {
          expect(store.canCreate).to.be.false;
        });

        it('returns true when selected', () => {
          store.selectGethAccount(GETH_ADDRESSES[0]);
          expect(store.canCreate).to.be.true;
        });
      });

      describe('createType === fromJSON/fromPresale', () => {
        beforeEach(() => {
          store.setCreateType('fromJSON');
        });

        it('returns true on no errors', () => {
          expect(store.canCreate).to.be.true;
        });

        it('returns false on nameError', () => {
          store.setName('');
          expect(store.canCreate).to.be.false;
        });

        it('returns false on walletFileError', () => {
          store.setWalletFile('testing');
          expect(store.canCreate).to.be.false;
        });
      });

      describe('createType === fromNew', () => {
        beforeEach(() => {
          store.setCreateType('fromNew');
        });

        it('returns true on no errors', () => {
          expect(store.canCreate).to.be.true;
        });

        it('returns false on nameError', () => {
          store.setName('');
          expect(store.canCreate).to.be.false;
        });

        it('returns false on passwordRepeatError', () => {
          store.setPassword('testing');
          expect(store.canCreate).to.be.false;
        });
      });

      describe('createType === fromPhrase', () => {
        beforeEach(() => {
          store.setCreateType('fromPhrase');
        });

        it('returns true on no errors', () => {
          expect(store.canCreate).to.be.true;
        });

        it('returns false on nameError', () => {
          store.setName('');
          expect(store.canCreate).to.be.false;
        });

        it('returns false on passwordRepeatError', () => {
          store.setPassword('testing');
          expect(store.canCreate).to.be.false;
        });
      });

      describe('createType === fromRaw', () => {
        beforeEach(() => {
          store.setCreateType('fromRaw');
        });

        it('returns true on no errors', () => {
          expect(store.canCreate).to.be.true;
        });

        it('returns false on nameError', () => {
          store.setName('');
          expect(store.canCreate).to.be.false;
        });

        it('returns false on passwordRepeatError', () => {
          store.setPassword('testing');
          expect(store.canCreate).to.be.false;
        });

        it('returns false on rawKeyError', () => {
          store.setRawKey('testing');
          expect(store.canCreate).to.be.false;
        });
      });

      describe('createType === anythingElse', () => {
        beforeEach(() => {
          store.setCreateType('anythingElse');
        });

        it('always returns false', () => {
          expect(store.canCreate).to.be.false;
        });
      });
    });

    describe('passwordRepeatError', () => {
      it('is clear when passwords match', () => {
        store.setPassword('testing');
        store.setPasswordRepeat('testing');
        expect(store.passwordRepeatError).to.be.null;
      });

      it('has error when passwords does not match', () => {
        store.setPassword('testing');
        store.setPasswordRepeat('testing2');
        expect(store.passwordRepeatError).not.to.be.null;
      });
    });
  });

  describe('operations', () => {
    describe('createAccount', () => {
      let createAccountFromGethSpy;
      let createAccountFromWalletSpy;
      let createAccountFromPhraseSpy;
      let createAccountFromRawSpy;
      let busySpy;

      beforeEach(() => {
        createAccountFromGethSpy = sinon.spy(store, 'createAccountFromGeth');
        createAccountFromWalletSpy = sinon.spy(store, 'createAccountFromWallet');
        createAccountFromPhraseSpy = sinon.spy(store, 'createAccountFromPhrase');
        createAccountFromRawSpy = sinon.spy(store, 'createAccountFromRaw');
        busySpy = sinon.spy(store, 'setBusy');
      });

      afterEach(() => {
        store.createAccountFromGeth.restore();
        store.createAccountFromWallet.restore();
        store.createAccountFromPhrase.restore();
        store.createAccountFromRaw.restore();
        store.setBusy.restore();
      });

      it('throws error on invalid createType', () => {
        store.setCreateType('testing');
        expect(() => store.createAccount()).to.throw;
      });

      it('calls createAccountFromGeth on createType === fromGeth', () => {
        store.setCreateType('fromGeth');

        return store.createAccount().then(() => {
          expect(createAccountFromGethSpy).to.have.been.called;
        });
      });

      it('calls createAccountFromWallet on createType === fromJSON', () => {
        store.setCreateType('fromJSON');

        return store.createAccount().then(() => {
          expect(createAccountFromWalletSpy).to.have.been.called;
        });
      });

      it('calls createAccountFromPhrase on createType === fromNew', () => {
        store.setCreateType('fromNew');

        return store.createAccount().then(() => {
          expect(createAccountFromPhraseSpy).to.have.been.called;
        });
      });

      it('calls createAccountFromPhrase on createType === fromPhrase', () => {
        store.setCreateType('fromPhrase');

        return store.createAccount().then(() => {
          expect(createAccountFromPhraseSpy).to.have.been.called;
        });
      });

      it('calls createAccountFromWallet on createType === fromPresale', () => {
        store.setCreateType('fromPresale');

        return store.createAccount().then(() => {
          expect(createAccountFromWalletSpy).to.have.been.called;
        });
      });

      it('calls createAccountFromRaw on createType === fromRaw', () => {
        store.setCreateType('fromRaw');

        return store.createAccount().then(() => {
          expect(createAccountFromRawSpy).to.have.been.called;
        });
      });

      it('moves account to vault when vaultName set', () => {
        store.setCreateType('fromNew');
        store.setVaultName('testing');

        return store.createAccount(vaultStore).then(() => {
          expect(vaultStore.moveAccount).to.have.been.calledWith('testing', ADDRESS);
        });
      });

      it('sets and rests the busy flag', () => {
        store.setCreateType('fromNew');

        return store.createAccount().then(() => {
          expect(busySpy).to.have.been.calledWith(true);
          expect(busySpy).to.have.been.calledWith(false);
        });
      });

      describe('createAccountFromGeth', () => {
        beforeEach(() => {
          store.selectGethAccount(GETH_ADDRESSES[0]);
        });

        it('calls parity.importGethAccounts', () => {
          return store.createAccountFromGeth().then(() => {
            expect(store._api.parity.importGethAccounts).to.have.been.calledWith([GETH_ADDRESSES[0]]);
          });
        });

        it('sets the account name', () => {
          return store.createAccountFromGeth().then(() => {
            expect(store._api.parity.setAccountName).to.have.been.calledWith(GETH_ADDRESSES[0], 'Geth Import');
          });
        });

        it('sets the account meta', () => {
          return store.createAccountFromGeth(-1).then(() => {
            expect(store._api.parity.setAccountMeta).to.have.been.calledWith(GETH_ADDRESSES[0], {
              timestamp: -1
            });
          });
        });
      });

      describe('createAccountFromPhrase', () => {
        beforeEach(() => {
          store.setCreateType('fromPhrase');
          store.setName('some name');
          store.setPassword('P@55worD');
          store.setPasswordHint('some hint');
          store.setPhrase('some phrase');
        });

        it('calls parity.newAccountFromWallet', () => {
          return store.createAccountFromPhrase().then(() => {
            expect(store._api.parity.newAccountFromPhrase).to.have.been.calledWith('some phrase', 'P@55worD');
          });
        });

        it('sets the address', () => {
          return store.createAccountFromPhrase().then(() => {
            expect(store.address).to.equal(ADDRESS);
          });
        });

        it('sets the account name', () => {
          return store.createAccountFromPhrase().then(() => {
            expect(store._api.parity.setAccountName).to.have.been.calledWith(ADDRESS, 'some name');
          });
        });

        it('sets the account meta', () => {
          return store.createAccountFromPhrase(-1).then(() => {
            expect(store._api.parity.setAccountMeta).to.have.been.calledWith(ADDRESS, {
              passwordHint: 'some hint',
              timestamp: -1
            });
          });
        });

        it('adjusts phrases for Windows', () => {
          store.setWindowsPhrase(true);
          return store.createAccountFromPhrase().then(() => {
            expect(store._api.parity.newAccountFromPhrase).to.have.been.calledWith('some\r phrase\r', 'P@55worD');
          });
        });

        it('adjusts phrases for Windows (except last word)', () => {
          store.setWindowsPhrase(true);
          store.setPhrase('misjudged phrase');
          return store.createAccountFromPhrase().then(() => {
            expect(store._api.parity.newAccountFromPhrase).to.have.been.calledWith('misjudged phrase\r', 'P@55worD');
          });
        });
      });

      describe('createAccountFromRaw', () => {
        beforeEach(() => {
          store.setName('some name');
          store.setPassword('P@55worD');
          store.setPasswordHint('some hint');
          store.setRawKey('rawKey');
        });

        it('calls parity.newAccountFromSecret', () => {
          return store.createAccountFromRaw().then(() => {
            expect(store._api.parity.newAccountFromSecret).to.have.been.calledWith('rawKey', 'P@55worD');
          });
        });

        it('sets the address', () => {
          return store.createAccountFromRaw().then(() => {
            expect(store.address).to.equal(ADDRESS);
          });
        });

        it('sets the account name', () => {
          return store.createAccountFromRaw().then(() => {
            expect(store._api.parity.setAccountName).to.have.been.calledWith(ADDRESS, 'some name');
          });
        });

        it('sets the account meta', () => {
          return store.createAccountFromRaw(-1).then(() => {
            expect(store._api.parity.setAccountMeta).to.have.been.calledWith(ADDRESS, {
              passwordHint: 'some hint',
              timestamp: -1
            });
          });
        });
      });

      describe('createAccountFromWallet', () => {
        beforeEach(() => {
          store.setName('some name');
          store.setPassword('P@55worD');
          store.setPasswordHint('some hint');
          store.setWalletJson('json');
        });

        it('calls parity.newAccountFromWallet', () => {
          return store.createAccountFromWallet().then(() => {
            expect(store._api.parity.newAccountFromWallet).to.have.been.calledWith('json', 'P@55worD');
          });
        });

        it('sets the address', () => {
          return store.createAccountFromWallet().then(() => {
            expect(store.address).to.equal(ADDRESS);
          });
        });

        it('sets the account name', () => {
          return store.createAccountFromWallet().then(() => {
            expect(store._api.parity.setAccountName).to.have.been.calledWith(ADDRESS, 'some name');
          });
        });

        it('sets the account meta', () => {
          return store.createAccountFromWallet(-1).then(() => {
            expect(store._api.parity.setAccountMeta).to.have.been.calledWith(ADDRESS, {
              passwordHint: 'some hint',
              timestamp: -1
            });
          });
        });
      });
    });

    describe('createIdentities', () => {
      it('creates calls parity.generateSecretPhrase', () => {
        return store.createIdentities().then(() => {
          expect(store._api.parity.generateSecretPhrase).to.have.been.called;
        });
      });

      it('returns a map of 7 accounts', () => {
        return store.createIdentities().then((accounts) => {
          expect(Object.keys(accounts).length).to.equal(7);
        });
      });

      it('creates accounts with an address & phrase', () => {
        return store.createIdentities().then((accounts) => {
          Object.keys(accounts).forEach((address) => {
            const account = accounts[address];

            expect(account.address).to.equal(address);
            expect(account.phrase).to.be.ok;
          });
        });
      });
    });

    describe('loadAvailableGethAccounts', () => {
      it('retrieves the list from parity.listGethAccounts', () => {
        return store.loadAvailableGethAccounts().then(() => {
          expect(store._api.parity.listGethAccounts).to.have.been.called;
        });
      });

      it('sets the available addresses with balances', () => {
        return store.loadAvailableGethAccounts().then(() => {
          expect(store.gethAccountsAvailable[0]).to.deep.equal({
            address: GETH_ADDRESSES[0],
            balance: '0.00000'
          });
        });
      });

      it('filters accounts already available', () => {
        return store.loadAvailableGethAccounts().then(() => {
          expect(store.gethAccountsAvailable.length).to.equal(1);
        });
      });
    });
  });
});
