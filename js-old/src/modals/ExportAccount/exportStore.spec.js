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

import ExportStore from './exportStore';

const ADDRESS = '0x00000123456789abcdef123456789abcdef123456789abcdef';
const ADDRESS_2 = '0x123456789abcdef123456789abcdef123456789abcdef00000';
const ACCOUNTS = { ADDRESS: {}, ADDRESS_2: {} };

let api;
let AccountStore;

function createApi () {
  return {
    eth: {
    },
    parity: {
      exportAccount: sinon.stub().resolves({})
    }
  };
}

function createMultiAccountStore (loadGeth) {
  api = createApi();
  AccountStore = new ExportStore(api, ACCOUNTS, null, null);

  return AccountStore;
}

describe('modals/exportAccount/Store', () => {
  beforeEach(() => {
    createMultiAccountStore();
  });

  describe('constructor', () => {
    it('insert api', () => {
      expect(AccountStore._api).to.deep.equal(api);
    });

    it('insert accounts', () => {
      expect(AccountStore.accounts).to.deep.equal(ACCOUNTS);
    });

    it('newError created', () => {
      expect(AccountStore._newError).to.deep.equal(null);
    });
  });

  describe('@action', () => {
    describe('toggleSelectedAccount', () => {
      it('Updates the selected accounts', () => {
        // First set selectedAccounts
        AccountStore.selectedAccounts = {
          [ADDRESS]: true,
          [ADDRESS_2]: false
        };
        // Toggle
        AccountStore.toggleSelectedAccount(ADDRESS_2);
        // Prep eqality
        const eq = {
          [ADDRESS]: true,
          [ADDRESS_2]: true
        };

        // Check equality
        expect(JSON.stringify(AccountStore.selectedAccounts)).to.deep.equal(JSON.stringify(eq));
      });
    });

    describe('getPassword', () => {
      it('Grab from the selected accounts input', () => {
        // First set passwordInputs
        AccountStore.passwordInputs = {
          [ADDRESS]: 'abc'
        };
        // getPassword
        const pass = AccountStore.getPassword(ADDRESS);

        // Check equality
        expect(AccountStore.passwordInputs[ADDRESS]).to.deep.equal(pass);
      });
    });

    describe('setPassword & getPassword', () => {
      it('First save the input of the selected account, than get the input.', () => {
        // Set password
        AccountStore.selectedAccount = ADDRESS;
        // Set new pass
        AccountStore.changePassword(null, 'abc');
        // getPassword
        const pass = AccountStore.getPassword(ADDRESS);

        // Check equality
        expect(AccountStore.passwordInputs[ADDRESS]).to.deep.equal(pass);
      });
    });

    describe('changePassword', () => {
      it('Change the stored value with the new input.', () => {
        // First set selectedAccounts
        AccountStore.selectedAccounts = {
          [ADDRESS]: true,
          [ADDRESS_2]: false
        };
        // First set passwordInputs
        AccountStore.passwordInputs = {
          [ADDRESS]: 'abc'
        };
        // 'Click' on the address:
        AccountStore.onClick(ADDRESS);
        // Change password
        AccountStore.changePassword(null, '123');
        // Check equality
        expect(AccountStore.passwordInputs[ADDRESS]).to.deep.equal('123');
      });
    });
  });
});
