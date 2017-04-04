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

import { keythereum } from '../ethkey';

export default class Account {
  constructor (persist, data) {
    const {
      keyObject,
      meta = {},
      name = ''
    } = data;

    this._persist = persist;
    this._keyObject = keyObject;
    this._name = name;
    this._meta = meta;
  }

  isValidPassword (password) {
    try {
      keythereum.recover(Buffer.from(password), this._keyObject);
      return true;
    } catch (e) {
      return false;
    }
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
    return this._keyObject.id;
  }

  decryptPrivateKey (password) {
    return keythereum.recover(Buffer.from(password), this._keyObject);
  }

  static fromPrivateKey (persist, key, password) {
    const iv = keythereum.crypto.randomBytes(16);
    const salt = keythereum.crypto.randomBytes(32);

    // Keythereum will fail if `password` is an empty string
    password = Buffer.from(password);

    const keyObject = keythereum.dump(password, key, salt, iv);

    const account = new Account(persist, { keyObject });

    return account;
  }

  toJSON () {
    return {
      keyObject: this._keyObject,
      name: this._name,
      meta: this._meta
    };
  }
}
