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

import { action, observable, transaction } from 'mobx';

let instance;

export default class Store {
  @observable isOpen = false;
  @observable listAll = [];
  @observable listOpened = [];

  constructor (api) {
    this._api = api;
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

  closeModal () {
    this.setOpen(false);
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
