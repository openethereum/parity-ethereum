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

import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import VaultAccounts from './';

const ACCOUNT_A = '0x1234567890123456789012345678901234567890';
const ACCOUNT_B = '0x0123456789012345678901234567890123456789';
const ACCOUNT_C = '0x9012345678901234567890123456789012345678';
const ACCOUNT_D = '0x8901234567890123456789012345678901234567';
const VAULTNAME = 'testVault';
const ACCOUNTS = {
  [ACCOUNT_A]: {
    address: ACCOUNT_A,
    uuid: null
  },
  [ACCOUNT_B]: {
    address: ACCOUNT_B,
    uuid: ACCOUNT_B,
    meta: {
      vault: 'somethingElse'
    }
  },
  [ACCOUNT_C]: {
    address: ACCOUNT_C,
    uuid: ACCOUNT_C,
    meta: {
      vault: VAULTNAME
    }
  },
  [ACCOUNT_D]: {
    address: ACCOUNT_D,
    uuid: ACCOUNT_D,
    meta: {
      vault: VAULTNAME
    }
  }
};

let api;
let component;
let instance;
let reduxStore;
let vaultStore;

function createApi () {
  api = {
    parity: {
      allAccountsInfo: sinon.stub().resolves({})
    }
  };

  return api;
}

function createReduxStore () {
  reduxStore = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        balances: {
          balances: {}
        },
        personal: {
          accounts: ACCOUNTS
        }
      };
    }
  };

  return reduxStore;
}

function createVaultStore () {
  vaultStore = {
    isBusyAccounts: false,
    isModalAccountsOpen: true,
    selectedAccounts: { [ACCOUNT_B]: true, [ACCOUNT_C]: true },
    vaultName: VAULTNAME,
    closeAccountsModal: sinon.stub(),
    moveAccounts: sinon.stub().resolves(true),
    toggleSelectedAccount: sinon.stub()
  };

  return vaultStore;
}

function render () {
  component = shallow(
    <VaultAccounts vaultStore={ createVaultStore() } />,
    {
      context: {
        store: createReduxStore()
      }
    }
  ).find('VaultAccounts').shallow({
    context: {
      api: createApi()
    }
  });
  instance = component.instance();

  return component;
}

describe('modals/VaultAccounts', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('SelectionList', () => {
      let sectionList;

      beforeEach(() => {
        sectionList = component.find('SelectionList');
      });

      it('has the filtered accounts', () => {
        expect(sectionList.props().items).to.deep.equal([
          ACCOUNTS[ACCOUNT_B], ACCOUNTS[ACCOUNT_C], ACCOUNTS[ACCOUNT_D]
        ]);
      });

      it('renders via renderAccount', () => {
        expect(sectionList.props().renderItem).to.equal(instance.renderAccount);
      });
    });
  });

  describe('event handlers', () => {
    describe('onClose', () => {
      beforeEach(() => {
        instance.onClose();
      });

      it('calls into closeAccountsModal', () => {
        expect(vaultStore.closeAccountsModal).to.have.been.called;
      });
    });

    describe('onExecute', () => {
      beforeEach(() => {
        sinon.spy(instance, 'onClose');
        return instance.onExecute();
      });

      afterEach(() => {
        instance.onClose.restore();
      });

      it('calls into moveAccounts', () => {
        expect(vaultStore.moveAccounts).to.have.been.calledWith(VAULTNAME, [ACCOUNT_B], [ACCOUNT_C]);
      });

      it('closes modal', () => {
        expect(instance.onClose).to.have.been.called;
      });
    });
  });
});
