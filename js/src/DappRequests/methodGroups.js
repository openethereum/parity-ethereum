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

const methodGroups = {
  shell: {
    methods: [
      'shell_getApps',
      'shell_getFilteredMethods',
      'shell_getMethodGroups',
      'shell_getMethodPermissions',
      'shell_setAppVisibility',
      'shell_setMethodPermissions'
    ]
  },
  accountsView: {
    methods: ['parity_accountsInfo', 'parity_allAccountsInfo']
  },
  accountsCreate: {
    methods: [
      'parity_generateSecretPhrase',
      'parity_importGethAccounts',
      'parity_listGethAccounts',
      'parity_newAccountFromPhrase',
      'parity_newAccountFromSecret',
      'parity_newAccountFromWallet',
      'parity_phraseToAddress'
    ]
  },
  accountsEdit: {
    methods: ['parity_setAccountName', 'parity_setAccountMeta']
  },
  upgrade: {
    methods: [
      'parity_consensusCapability',
      'parity_executeUpgrade',
      'parity_upgradeReady',
      'parity_versionInfo'
    ]
  },
  vaults: {
    methods: [
      'parity_changeVault',
      'parity_changeVaultPassword',
      'parity_closeVault',
      'parity_getVaultMeta',
      'parity_listVaults',
      'parity_listOpenedVaults',
      'parity_newVault',
      'parity_openVault',
      'parity_setVaultMeta'
    ]
  },
  other: {
    methods: [
      'parity_checkRequest',
      'parity_hashContent',
      'parity_localTransactions'
    ]
  }
};

const methodGroupFromMethod = {}; // Maps method to methodGroup

// Populate methodGroupFromMethod
Object.keys(methodGroups).forEach(groupId => {
  methodGroups[groupId].methods.forEach(method => {
    methodGroupFromMethod[method] = groupId;
  });
});

export { methodGroupFromMethod };
export default methodGroups;
