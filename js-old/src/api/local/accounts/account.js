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

import { createKeyObject, decryptPrivateKey } from '../ethkey';

export default class Account {
  constructor (persist, data = {}) {
    const {
      keyObject = null,
      meta = {},
      name = ''
    } = data;

    this._persist = persist;
    this._keyObject = keyObject;
    this._name = name;
    this._meta = meta;
  }

  isValidPassword (password) {
    if (!this._keyObject) {
      return false;
    }

    return decryptPrivateKey(this._keyObject, password)
      .then((privateKey) => {
        if (!privateKey) {
          return false;
        }

        return true;
      });
  }

  export () {
    const exported = Object.assign({}, this._keyObject);

    exported.meta = JSON.stringify(this._meta);
    exported.name = this._name;

    return exported;
  }

  get address () {
    return `0x${this._keyObject.address.toLowerCase()}`;
  }

  get name () {
    return this._name;
  }

  set name (name) {
    this._name = name;

    this._persist();
  }

  get meta () {
    return JSON.stringify(this._meta);
  }

  set meta (meta) {
    this._meta = JSON.parse(meta);

    this._persist();
  }

  get uuid () {
    if (!this._keyObject) {
      return null;
    }

    return this._keyObject.id;
  }

  decryptPrivateKey (password) {
    return decryptPrivateKey(this._keyObject, password);
  }

  changePassword (key, password) {
    return createKeyObject(key, password).then((keyObject) => {
      this._keyObject = keyObject;

      this._persist();
    });
  }

  static fromPrivateKey (persist, key, password) {
    return createKeyObject(key, password).then((keyObject) => {
      const account = new Account(persist, { keyObject });

      return account;
    });
  }

  toJSON () {
    return {
      keyObject: this._keyObject,
      name: this._name,
      meta: this._meta
    };
  }
}
