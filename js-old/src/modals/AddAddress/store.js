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

import { action, computed, transaction, observable } from 'mobx';

import { ERRORS, validateAddress, validateName } from '~/util/validation';

export default class Store {
  @observable address = '';
  @observable addressError = ERRORS.invalidAddress;
  @observable createError = null;
  @observable description = '';
  @observable name = '';
  @observable nameError = ERRORS.invalidName;

  constructor (api, contacts) {
    this._api = api;
    this._contacts = contacts;
  }

  @computed get hasError () {
    return !!(this.addressError || this.nameError);
  }

  @action setAddress = (_address) => {
    let { address, addressError } = validateAddress(_address);

    if (!addressError) {
      const contact = this._contacts[address];

      if (contact) {
        addressError = ERRORS.duplicateAddress;
      }
    }

    transaction(() => {
      this.address = address;
      this.addressError = addressError;
    });
  }

  @action setCreateError = (error) => {
    this.createError = error;
  }

  @action setDescription = (description) => {
    this.description = description;
  }

  @action setName = (_name) => {
    const { name, nameError } = validateName(_name);

    transaction(() => {
      this.name = name;
      this.nameError = nameError;
    });
  }

  add () {
    return Promise
      .all([
        this._api.parity.setAccountName(this.address, this.name),
        this._api.parity.setAccountMeta(this.address, {
          description: this.description,
          timestamp: Date.now(),
          deleted: false
        })
      ])
      .catch((error) => {
        console.warn('Store:add', error);
        this.setCreateError(error);
      });
  }
}
