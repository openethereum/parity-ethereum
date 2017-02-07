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
import ERRORS from '../CreateAccount/errors';

let instance;

export default class Store {
  @observable createName = '';
  @observable createNameError = ERRORS.noName;
  @observable createPassword = '';
  @observable createPasswordHint = '';
  @observable createPasswordRepeat = '';
  @observable isOpen = false;
  @observable isOpenAdd = false;
  @observable listAll = [];
  @observable listOpened = [];

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
      this.createPasswordRepeatError = null;
    });
  }

  @action setCreateName = (name) => {
    let nameError = null;

    if (!name || !name.trim().length) {
      nameError = ERRORS.noName;
    } else if (this.listAll.includes(name)) {
      nameError = ERRORS.duplicateName;
    }

    transaction(() => {
      this.createName = name;
      this.createNameError = nameError;
    });
  }

  @action setListAll = (listAll) => {
    this.listAll = listAll;
  }

  @action setListOpened = (listOpened) => {
    this.listOpened = listOpened;
  }

  @action setOpen = (isOpen) => {
    this.isOpen = isOpen;
  }

  @action setOpenAdd = (isOpenAdd) => {
    this.isOpenAdd = isOpenAdd;
  }

  closeAdd () {
    this.setOpenAdd(false);
  }

  closeModal () {
    this.setOpen(false);
  }

  openAdd () {
    this.clearCreateFields();
    this.setOpenAdd(true);
  }

  openModal () {
    this.setOpen(true);

    return this.loadVaults();
  }

  loadVaults () {
    return Promise
      .all([
        this._api.parity.listVaults(),
        this._api.parity.listOpenedVaults()
      ])
      .then(([allVaults, openedVaults]) => {
        transaction(() => {
          this.setListAll(allVaults);
          this.setListOpened(openedVaults);
        });
      })
      .catch((error) => {
        console.warn('loadVaults', error);
      });
  }

  static get (api) {
    if (!instance) {
      instance = new Store(api);
    }

    return instance;
  }
}
