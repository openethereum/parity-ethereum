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

import { action, computed, observable, transaction } from 'mobx';

// TODO: We need to move this to a generic location, it should most probably be
// merged with the other valitation errors. Import here better than duplication.
import ERRORS from '@parity/dapp-accounts/CreateAccount/errors';

let instance;

export default class Store {
  @observable isBusyAccounts = false;
  @observable isBusyCreate = false;
  @observable isBusyLoad = false;
  @observable isBusyLock = false;
  @observable isBusyMeta = false;
  @observable isBusyUnlock = false;
  @observable isModalAccountsOpen = false;
  @observable isModalCreateOpen = false;
  @observable isModalLockOpen = false;
  @observable isModalMetaOpen = false;
  @observable isModalUnlockOpen = false;
  @observable selectedAccounts = {};
  @observable vault = null;
  @observable vaults = [];
  @observable vaultsOpened = [];
  @observable vaultNames = [];
  @observable vaultName = '';
  @observable vaultNameError = ERRORS.noName;
  @observable vaultDescription = '';
  @observable vaultPassword = '';
  @observable vaultPasswordHint = '';
  @observable vaultPasswordOld = '';
  @observable vaultPasswordRepeat = '';
  @observable vaultTags = [];

  constructor (api) {
    this._api = api;
  }

  @computed get vaultPasswordRepeatError () {
    return this.vaultPassword === this.vaultPasswordRepeat
      ? null
      : ERRORS.noMatchPassword;
  }

  @action clearVaultFields = () => {
    transaction(() => {
      this.setVaultName('');
      this.setVaultDescription('');
      this.setVaultPassword('');
      this.setVaultPasswordHint('');
      this.setVaultPasswordOld('');
      this.setVaultPasswordRepeat('');
      this.setVaultTags([]);
    });
  }

  @action setBusyAccounts = (isBusy) => {
    this.isBusyAccounts = isBusy;
  }

  @action setBusyCreate = (isBusy) => {
    this.isBusyCreate = isBusy;
  }

  @action setBusyLoad = (isBusy) => {
    this.isBusyLoad = isBusy;
  }

  @action setBusyLock = (isBusy) => {
    this.isBusyLock = isBusy;
  }

  @action setBusyMeta = (isBusy) => {
    this.isBusyMeta = isBusy;
  }

  @action setBusyUnlock = (isBusy) => {
    this.isBusyUnlock = isBusy;
  }

  @action setModalAccountsOpen = (isOpen) => {
    transaction(() => {
      this.setBusyAccounts(false);
      this.isModalAccountsOpen = isOpen;
    });
  }

  @action setModalCreateOpen = (isOpen) => {
    transaction(() => {
      this.setBusyCreate(false);
      this.isModalCreateOpen = isOpen;
    });
  }

  @action setModalLockOpen = (isOpen) => {
    transaction(() => {
      this.setBusyLock(false);
      this.isModalLockOpen = isOpen;
    });
  }

  @action setModalMetaOpen = (isOpen) => {
    transaction(() => {
      this.setBusyMeta(false);
      this.isModalMetaOpen = isOpen;
    });
  }

  @action setModalUnlockOpen = (isOpen) => {
    transaction(() => {
      this.setBusyUnlock(false);
      this.setVaultPassword('');
      this.isModalUnlockOpen = isOpen;
    });
  }

  @action setSelectedAccounts = (selectedAccounts) => {
    this.selectedAccounts = selectedAccounts;
  }

  @action setVaults = (allVaults, openedVaults, metaData) => {
    transaction(() => {
      this.vaultNames = allVaults.map((name) => name.toLowerCase());
      this.vaults = allVaults.map((name, index) => {
        return {
          meta: metaData[index] || {},
          name,
          isOpen: openedVaults.includes(name)
        };
      });
      this.vaultsOpened = this.vaults.filter((vault) => vault.isOpen);
    });
  }

  @action setVaultDescription = (description) => {
    this.vaultDescription = description;
  }

  @action setVaultName = (name) => {
    let nameError = null;

    if (!name || !name.trim().length) {
      nameError = ERRORS.noName;
    } else {
      const lowerName = name.toLowerCase();

      if (this.vaultNames.includes(lowerName)) {
        nameError = ERRORS.duplicateName;
      }
    }

    transaction(() => {
      this.vault = this.vaults.find((vault) => vault.name === name);
      this.vaultName = name;
      this.vaultNameError = nameError;
    });
  }

  @action setVaultPassword = (password) => {
    this.vaultPassword = password;
  }

  @action setVaultPasswordHint = (hint) => {
    this.vaultPasswordHint = hint;
  }

  @action setVaultPasswordOld = (password) => {
    this.vaultPasswordOld = password;
  }

  @action setVaultPasswordRepeat = (password) => {
    this.vaultPasswordRepeat = password;
  }

  @action setVaultTags = (tags) => {
    this.vaultTags = tags;
  }

  @action toggleSelectedAccount = (address) => {
    this.setSelectedAccounts(Object.assign({}, this.selectedAccounts, {
      [address]: !this.selectedAccounts[address] })
    );
  }

  closeAccountsModal () {
    this.setModalAccountsOpen(false);
  }

  closeCreateModal () {
    this.setModalCreateOpen(false);
  }

  closeLockModal () {
    this.setModalLockOpen(false);
  }

  closeMetaModal () {
    this.setModalMetaOpen(false);
  }

  closeUnlockModal () {
    this.setModalUnlockOpen(false);
  }

  openAccountsModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setSelectedAccounts({});
      this.setModalAccountsOpen(true);
    });
  }

  openCreateModal () {
    transaction(() => {
      this.clearVaultFields();
      this.setModalCreateOpen(true);
    });
  }

  openLockModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setModalLockOpen(true);
    });
  }

  openMetaModal (name) {
    transaction(() => {
      this.clearVaultFields();
      this.setVaultName(name);

      if (this.vault && this.vault.meta) {
        this.setVaultDescription(this.vault.meta.description);
        this.setVaultPasswordHint(this.vault.meta.passwordHint);
      }

      this.setModalMetaOpen(true);
    });
  }

  openUnlockModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setModalUnlockOpen(true);
    });
  }

  loadVaults = () => {
    this.setBusyLoad(true);

    return Promise
      .all([
        this._api.parity.listVaults(),
        this._api.parity.listOpenedVaults()
      ])
      .then(([allVaults, openedVaults]) => {
        return Promise
          .all(allVaults.map((name) => this._api.parity.getVaultMeta(name)))
          .then((metaData) => {
            transaction(() => {
              this.setBusyLoad(false);
              this.setVaults(allVaults, openedVaults, metaData);
            });
          });
      })
      .catch((error) => {
        console.warn('loadVaults', error);
        this.setBusyLoad(false);
      });
  }

  closeVault () {
    this.setBusyLock(true);

    return this._api.parity
      .closeVault(this.vaultName)
      .then(this.loadVaults)
      .then(() => {
        this.setBusyLock(false);
      })
      .catch((error) => {
        console.error('closeVault', error);
        this.setBusyLock(false);
        throw error;
      });
  }

  createVault () {
    if (this.vaultNameError || this.vaultPasswordRepeatError) {
      return Promise.reject();
    }

    this.setBusyCreate(true);

    return this._api.parity
      .newVault(this.vaultName, this.vaultPassword)
      .then(() => {
        return this._api.parity.setVaultMeta(this.vaultName, {
          description: this.vaultDescription,
          passwordHint: this.vaultPasswordHint,
          tags: this.vaultTags
        });
      })
      .then(this.loadVaults)
      .then(() => {
        this.setBusyCreate(false);
      })
      .catch((error) => {
        console.error('createVault', error);
        this.setBusyCreate(false);
        throw error;
      });
  }

  editVaultMeta () {
    this.setBusyMeta(true);

    return this._api.parity
      .setVaultMeta(this.vaultName, {
        description: this.vaultDescription,
        passwordHint: this.vaultPasswordHint,
        tags: this.vaultTags
      })
      .then(this.loadVaults)
      .then(() => {
        this.setBusyMeta(false);
      })
      .catch((error) => {
        console.error('editVaultMeta', error);
        this.setBusyMeta(false);
        throw error;
      });
  }

  editVaultPassword () {
    this.setBusyMeta(true);

    return this._api.parity
      .closeVault(this.vaultName)
      .then(() => {
        return this._api.parity.openVault(this.vaultName, this.vaultPasswordOld);
      })
      .then(() => {
        return this._api.parity.changeVaultPassword(this.vaultName, this.vaultPassword);
      })
      .then(() => {
        this.setBusyMeta(false);
      })
      .catch((error) => {
        console.error('editVaultPassword', error);
        this.loadVaults();
        this.setBusyMeta(false);
        throw new Error('Unable to change the vault password');
      });
  }

  openVault () {
    this.setBusyUnlock(true);

    return this._api.parity
      .openVault(this.vaultName, this.vaultPassword)
      .then(this.loadVaults)
      .then(() => {
        this.setBusyUnlock(false);
      })
      .catch((error) => {
        console.error('openVault', error);
        this.setBusyUnlock(false);
        throw error;
      });
  }

  moveAccounts (vaultName, inAccounts, outAccounts) {
    this.setBusyAccounts(true);

    return Promise
      .all([
        inAccounts.map((address) => this._api.parity.changeVault(address, vaultName)),
        outAccounts.map((address) => this._api.parity.changeVault(address, ''))
      ])
      .then(this.loadVaults)
      .then(() => {
        this.setBusyAccounts(false);
      })
      .catch((error) => {
        console.error('moveAccounts', error);
        this.setBusyAccounts(false);
        throw error;
      });
  }

  moveAccount (vaultName, address) {
    this.setBusyAccounts(true);

    return this._api.parity
      .changeVault(address, vaultName)
      .then(this.loadVaults)
      .then(() => {
        this.setBusyAccounts(false);
      })
      .catch((error) => {
        console.error('moveAccount', error);
        this.setBusyAccounts(false);
        throw error;
      });
  }

  static get (api) {
    if (!instance) {
      instance = new Store(api);
    }

    return instance;
  }
}
