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

import { validateName } from '@parity/shared/util/validation';

export default class Store {
  @observable address = null;
  @observable isAccount = false;
  @observable isBusy = false;
  @observable description = null;
  @observable meta = null;
  @observable name = null;
  @observable nameError = null;
  @observable passwordHint = null;
  @observable tags = null;
  @observable vaultName = null;

  constructor (api, account) {
    const { address, name, meta, uuid } = account;

    this._api = api;

    transaction(() => {
      this.address = address;
      this.meta = meta || {};
      this.name = name || '';
      this.isAccount = !!uuid;

      this.description = this.meta.description || '';
      this.passwordHint = this.meta.passwordHint || '';
      this.tags = this.meta.tags && this.meta.tags.slice() || [];
      this.vaultName = this.meta.vault;
    });
  }

  @computed get hasError () {
    return !!(this.nameError);
  }

  @action setDescription = (description) => {
    this.description = description;
  }

  @action setName = (_name) => {
    const { name, nameError } = validateName(_name);

    transaction(() => {
      this.name = name;
      this.setNameError(nameError);
    });
  }

  @action setNameError = (nameError) => {
    this.nameError = nameError;
  }

  @action setPasswordHint = (passwordHint) => {
    this.passwordHint = passwordHint;
  }

  @action setBusy = (isBusy) => {
    this.isBusy = isBusy;
  }

  @action setTags = (tags) => {
    this.tags = tags.slice();
  }

  @action setVaultName = (vaultName) => {
    this.vaultName = vaultName;
  }

  save (vaultStore) {
    this.setBusy(true);

    const meta = {
      description: this.description,
      tags: this.tags.peek()
    };

    if (this.isAccount) {
      meta.passwordHint = this.passwordHint;
    }

    return Promise
      .all([
        this._api.parity.setAccountName(this.address, this.name),
        this._api.parity.setAccountMeta(this.address, Object.assign({}, this.meta, meta))
      ])
      .then(() => {
        if (vaultStore && this.isAccount && (this.meta.vault !== this.vaultName)) {
          return vaultStore.moveAccount(this.vaultName, this.address);
        }

        return true;
      })
      .then(() => {
        this.setBusy(false);
      })
      .catch((error) => {
        console.error('onSave', error);
        this.setBusy(false);
        throw error;
      });
  }
}
