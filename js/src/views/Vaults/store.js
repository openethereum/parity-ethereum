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
import ERRORS from '~/modals/CreateAccount/errors';

let instance;

export default class Store {
  @observable createName = '';
  @observable createNameError = ERRORS.noName;
  @observable createPassword = '';
  @observable createPasswordHint = '';
  @observable createPasswordRepeat = '';
  @observable isBusyAccounts = false;
  @observable isBusyClose = false;
  @observable isBusyCreate = false;
  @observable isBusyLoad = false;
  @observable isBusyOpen = false;
  @observable isModalAccountsOpen = false;
  @observable isModalCloseOpen = false;
  @observable isModalCreateOpen = false;
  @observable isModalOpenOpen = false;
  @observable vaults = [];
  @observable vaultNames = [];
  @observable vaultName = '';
  @observable vaultPassword = '';

  constructor (api) {
    this._api = api;
  }

  @computed get createPasswordRepeatError () {
    return this.createPassword === this.createPasswordRepeat
      ? null
      : ERRORS.noMatchPassword;
  }

  @action clearCreateFields = () => {
    transaction(() => {
      this.createName = '';
      this.createNameError = ERRORS.noName;
      this.createPassword = '';
      this.createPasswordHint = '';
      this.createPasswordRepeat = '';
    });
  }

  @action setBusyAccounts = (isBusy) => {
    this.isBusyAccounts = isBusy;
  }

  @action setBusyClose = (isBusy) => {
    this.isBusyClose = isBusy;
  }

  @action setBusyCreate = (isBusy) => {
    this.isBusyCreate = isBusy;
  }

  @action setBusyLoad = (isBusy) => {
    this.isBusyLoad = isBusy;
  }

  @action setBusyOpen = (isBusy) => {
    this.isBusyOpen = isBusy;
  }

  @action setCreateName = (name) => {
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
      this.createName = name;
      this.createNameError = nameError;
    });
  }

  @action setCreatePassword = (password) => {
    this.createPassword = password;
  }

  @action setCreatePasswordHint = (hint) => {
    this.createPasswordHint = hint;
  }

  @action setCreatePasswordRepeat = (password) => {
    this.createPasswordRepeat = password;
  }

  @action setModalAccountsOpen = (isOpen) => {
    this.isModalAccountsOpen = isOpen;
  }

  @action setModalCloseOpen = (isOpen) => {
    this.isModalCloseOpen = isOpen;
  }

  @action setModalCreateOpen = (isOpen) => {
    this.isModalCreateOpen = isOpen;
  }

  @action setModalOpenOpen = (isOpen) => {
    transaction(() => {
      this.setVaultPassword('');
      this.isModalOpenOpen = isOpen;
    });
  }

  @action setVaults = (allVaults, openedVaults) => {
    transaction(() => {
      this.vaultNames = allVaults.map((name) => name.toLowerCase());
      this.vaults = allVaults.map((name) => {
        return {
          name,
          isOpen: openedVaults.includes(name)
        };
      });
    });
  }

  @action setVaultName = (name) => {
    this.vaultName = name;
  }

  @action setVaultPassword = (password) => {
    this.vaultPassword = password;
  }

  closeAccountsModal () {
    this.setModalAccountsOpen(false);
  }

  closeCloseModal () {
    this.setModalCloseOpen(false);
  }

  closeCreateModal () {
    this.setModalCreateOpen(false);
  }

  closeOpenModal () {
    this.setModalOpenOpen(false);
  }

  openAccountsModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setModalAccountsOpen(true);
    });
  }

  openCloseModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setModalCloseOpen(true);
    });
  }

  openCreateModal () {
    transaction(() => {
      this.clearCreateFields();
      this.setModalCreateOpen(true);
    });
  }

  openOpenModal (name) {
    transaction(() => {
      this.setVaultName(name);
      this.setModalOpenOpen(true);
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
        this.setBusyLoad(false);
        this.setVaults(allVaults, openedVaults);
      })
      .catch((error) => {
        console.error('loadVaults', error);
        this.setBusyLoad(false);
        throw error;
      });
  }

  closeVault () {
    this.setBusyClose(true);

    return this._api.parity
      .closeVault(this.vaultName)
      .then(() => {
        this.setBusyClose(false);
        return this.loadVaults();
      })
      .catch((error) => {
        console.error('closeVault', error);
        this.setBusyClose(false);
        throw error;
      });
  }

  createVault () {
    if (this.createNameError || this.createPasswordRepeatError) {
      return Promise.reject();
    }

    this.setBusyCreate(true);

    return this._api.parity
      .newVault(this.createName, this.createPassword)
      .then(() => {
        this.setBusyCreate(false);
        return this.loadVaults();
      })
      .catch((error) => {
        console.error('createVault', error);
        this.setBusyCreate(false);
        throw error;
      });
  }

  openVault () {
    this.setBusyOpen(true);

    return this._api.parity
      .openVault(this.vaultName, this.vaultPassword)
      .then(() => {
        this.setBusyOpen(false);
        return this.loadVaults();
      })
      .catch((error) => {
        console.error('openVault', error);
        this.setBusyOpen(false);
        throw error;
      });
  }

  moveAccounts (accounts) {
    this.setBusyAccounts(true);

    return Promise
      .all(accounts.map((address) => this._api.parity.changeVault(address, this.vaultName)))
      .then(() => {
        this.setBusyAccounts(false);
        return this.loadVaults();
      })
      .catch((error) => {
        console.error('moveAccounts', error);
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
