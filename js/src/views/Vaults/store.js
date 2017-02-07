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
  @observable isModalCloseOpen = false;
  @observable isModalCreateOpen = false;
  @observable isModalOpenOpen = false;
  @observable vaults = [];
  @observable vaultNames = [];
  @observable vaultName = null;
  @observable vaultPassword = null;

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

  @action setCreateName = (name) => {
    let nameError = null;

    if (!name || !name.trim().length) {
      nameError = ERRORS.noName;
    } else if (this.vaultNames.includes(name)) {
      nameError = ERRORS.duplicateName;
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

  @action setModalCloseOpen = (isOpen) => {
    this.isModalCloseOpen = isOpen;
  }

  @action setModalCreateOpen = (isOpen) => {
    this.isModalCreateOpen = isOpen;
  }

  @action setModalOpenOpen = (isOpen) => {
    this.isModalOpenOpen = isOpen;
  }

  @action setVaults = (allVaults, openedVaults) => {
    transaction(() => {
      this.vaultNames = allVaults;
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

  closeCloseModal () {
    this.setModalCloseOpen(false);
  }

  closeCreateModal () {
    this.setModalCreateOpen(false);
  }

  closeOpenModal () {
    this.setModalOpenOpen(false);
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
    return Promise
      .all([
        this._api.parity.listVaults(),
        this._api.parity.listOpenedVaults()
      ])
      .then(([allVaults, openedVaults]) => {
        this.setVaults(allVaults, openedVaults);
      })
      .catch((error) => {
        console.warn('loadVaults', error);
      });
  }

  closeVault () {
    return this._api.parity
      .closeVault(this.vaultName)
      .then(this.loadVaults)
      .catch((error) => {
        console.warn('closeVault', error);
      });
  }

  createVault () {
    if (this.createNameError || this.createPasswordRepeatError) {
      return Promise.reject();
    }

    return this._api.parity
      .newVault(this.createName, this.createPassword)
      .then(this.loadVaults)
      .catch((error) => {
        console.warn('createVault', error);
      });
  }

  openVault () {
    return this._api.parity
      .Vault(this.vaultName, this.vaultPassword)
      .then(this.loadVaults)
      .catch((error) => {
        console.warn('openVault', error);
      });
  }

  static get (api) {
    if (!instance) {
      instance = new Store(api);
    }

    return instance;
  }
}
