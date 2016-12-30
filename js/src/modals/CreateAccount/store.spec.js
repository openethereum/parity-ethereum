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

import { ACCOUNTS, GETH_ADDRESSES, createApi } from './createAccount.test.js';

let api;
let store;

function createStore () {
  api = createApi();
  store = new Store(api, ACCOUNTS);

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
  });

  describe('@action', () => {
    describe('clearErrors', () => {
      it('clears all errors', () => {
        store.clearErrors();

        expect(store.nameError).to.be.null;
        expect(store.passwordRepeatError).to.be.null;
        expect(store.rawKeyError).to.be.null;
        expect(store.walletFileError).to.be.null;
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
    describe('createIdentities', () => {
      it('creates calls parity.generateSecretPhrase', () => {
        return store.createIdentities().then(() => {
          expect(store._api.parity.generateSecretPhrase).to.have.been.called;
        });
      });

      it('returns a map of 5 accounts', () => {
        return store.createIdentities().then((accounts) => {
          expect(Object.keys(accounts).length).to.equal(5);
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
