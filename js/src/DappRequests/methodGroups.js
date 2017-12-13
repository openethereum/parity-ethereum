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
      'shell_loadApp'
    ]
  },
  dapps: {
    methods: [
      'parity_dappsRefresh',
      'parity_dappsUrl',
      'shell_getApps',
      'shell_getMethodPermissions'
    ]
  },
  dappsEdit: {
    methods: [
      'shell_setAppPinned',
      'shell_setAppVisibility',
      'shell_setMethodPermissions'
    ]
  },
  accounts: {
    methods: [
      'parity_accountsInfo',
      'parity_allAccountsInfo',
      'parity_getNewDappsAddresses',
      'parity_getNewDappsDefaultAddress',
      'parity_hardwareAccountsInfo',
      'parity_lockedHardwareAccountsInfo'
    ]
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
    methods: [
      'parity_setAccountName',
      'parity_setAccountMeta',
      'parity_hardwarePinMatrixAck',
      'parity_setNewDappsAddresses',
      'parity_setNewDappsDefaultAddress'
    ]
  },
  accountsDelete: {
    methods: [
      'parity_killAccount',
      'parity_removeAddress'
    ]
  },
  vaults: {
    methods: [
      'parity_closeVault',
      'parity_getVaultMeta',
      'parity_listVaults',
      'parity_listOpenedVaults',
      'parity_openVault'
    ]
  },
  vaultsCreate: {
    methods: [
      'parity_newVault'
    ]
  },
  vaultsEdit: {
    methods: [
      'parity_changeVault',
      'parity_changeVaultPassword',
      'parity_setVaultMeta'
    ]
  },
  signerRequests: {
    methods: [
      'parity_checkRequest',
      'parity_localTransactions'
    ]
  },
  signerConfirm: {
    methods: [
      'parity_confirmRequest',
      'parity_confirmRequestRaw',
      'parity_rejectRequest'
    ]
  },
  node: {
    methods: [
      'parity_hashContent',
      'parity_consensusCapability',
      'parity_upgradeReady',
      'parity_versionInfo',
      'parity_wsUrl'
    ]
  },
  nodeUpgrade: {
    methods: [
      'parity_executeUpgrade'
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
