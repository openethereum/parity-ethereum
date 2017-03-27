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
let multiAccountStore;
let oneAccountStore;

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
  multiAccountStore = new ExportStore(api, ACCOUNTS, null, null);

  return multiAccountStore;
}

function createOneAccountStore (loadGeth) {
  oneAccountStore = new ExportStore(api, ACCOUNTS, null, ADDRESS);

  return oneAccountStore;
}

describe('modals/exportAccount/Store', () => {
  beforeEach(() => {
    createMultiAccountStore();
    createOneAccountStore();
  });

  describe('constructor', () => {
    it('insert api', () => {
      expect(multiAccountStore._api).to.deep.equal(api);
    });

    it('insert accounts', () => {
      expect(multiAccountStore._accounts).to.deep.equal(ACCOUNTS);
    });

    it('insert address', () => {
      expect(multiAccountStore._address).to.deep.equal(null);
    });

    it('newError created', () => {
      expect(multiAccountStore._newError).to.deep.equal(null);
    });
  });

  describe('@action', () => {
    it('toggleSelectedAccount', () => {
      // First set selectedAccounts
      multiAccountStore.selectedAccounts = {
        [ADDRESS]: true,
        [ADDRESS_2]: false
      };
      // Toggle
      multiAccountStore.toggleSelectedAccount(ADDRESS_2);
      // Prep eqality
      const eq = {
        [ADDRESS]: false,
        [ADDRESS_2]: true
      };

      // Check equality
      expect(JSON.stringify(multiAccountStore.selectedAccounts)).to.deep.equal(JSON.stringify(eq));
    });

    it('getPassword', () => {
      // First set inputValue
      multiAccountStore.inputValue = {
        [ADDRESS]: 'abc'
      };
      // getPassword
      const pass = multiAccountStore.getPassword(ADDRESS);

      // Check equality
      expect(multiAccountStore.inputValue[ADDRESS]).to.deep.equal(pass);
    });

    it('setPassword & getPassword', () => {
      // First set pass
      multiAccountStore.setPassword(ADDRESS, 'abc');
      // getPassword
      const pass = multiAccountStore.getPassword(ADDRESS);

      // Check equality
      expect(multiAccountStore.inputValue[ADDRESS]).to.deep.equal(pass);
    });

    it('setPassword - oneAccount', () => {
      // First set pass
      oneAccountStore.setPassword(null, 'abc');
      // Check equality
      expect(oneAccountStore.accountValue).to.deep.equal('abc');
    });

    it('changePassword', () => {
      // First set selectedAccounts
      multiAccountStore.selectedAccounts = {
        [ADDRESS]: true,
        [ADDRESS_2]: false
      };
      // First set inputValue
      multiAccountStore.inputValue = {
        [ADDRESS]: 'abc'
      };
      // Change password
      multiAccountStore.changePassword(null, '123');
      // Check equality
      expect(multiAccountStore.inputValue[ADDRESS]).to.deep.equal('123');
    });

    it('changePassword - oneAccount', () => {
      // First set accountValue
      oneAccountStore.accountValue = 'abc';
      // Change password
      oneAccountStore.changePassword(null, '123');
      // Check equality
      expect(oneAccountStore.accountValue).to.deep.equal('123');
    });
  });
});
