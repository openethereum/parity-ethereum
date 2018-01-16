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

const ACCOUNT_A = '0x1234567890123456789012345678901234567890';
const ACCOUNT_B = '0x0123456789012345678901234567890123456789';
const ACCOUNT_C = '0x9012345678901234567890123456789012345678';
const ACCOUNT_D = '0x8901234567890123456789012345678901234567';

const TEST_VAULTS_ALL = ['vault1', 'vault2', 'vault3'];
const TEST_VAULTS_OPEN = ['vault2'];
const TEST_VAULTS_META = { something: 'test' };

const TEST_ACCOUNTS = {
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
      vault: 'test'
    }
  },
  [ACCOUNT_D]: {
    address: ACCOUNT_D,
    uuid: ACCOUNT_D,
    meta: {
      vault: 'test'
    }
  }
};

export function createApi () {
  return {
    parity: {
      listOpenedVaults: sinon.stub().resolves(TEST_VAULTS_OPEN),
      listVaults: sinon.stub().resolves(TEST_VAULTS_ALL),
      changeVault: sinon.stub().resolves(true),
      closeVault: sinon.stub().resolves(true),
      getVaultMeta: sinon.stub().resolves(TEST_VAULTS_META),
      newVault: sinon.stub().resolves(true),
      openVault: sinon.stub().resolves(true),
      setVaultMeta: sinon.stub().resolves(true),
      changeVaultPassword: sinon.stub().resolves(true)
    }
  };
}

export function createReduxStore () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        personal: {
          accounts: TEST_ACCOUNTS
        }
      };
    }
  };
}

export {
  TEST_ACCOUNTS,
  TEST_VAULTS_ALL,
  TEST_VAULTS_META,
  TEST_VAULTS_OPEN
};
