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

import Account from './account';
import localStore from 'store';
import { debounce } from 'lodash';

const NULL_ADDRESS = '0x0000000000000000000000000000000000000000';
const LS_STORE_KEY = '_parity::localAccounts';

export default class Accounts {
  constructor (data = localStore.get(LS_STORE_KEY) || {}) {
    const {
      last = NULL_ADDRESS,
      store = []
    } = data;

    this.persist = debounce(() => {
      localStore.set(LS_STORE_KEY, this);
    }, 100);

    this._last = last;
    this._store = store.map((data) => new Account(this.persist, data));
  }

  create (secret, password) {
    const privateKey = Buffer.from(secret.slice(2), 'hex');
    const account = Account.fromPrivateKey(this.persist, privateKey, password);

    this._store.push(account);
    this.lastAddress = account.address;

    this.persist();

    return account.address;
  }

  set lastAddress (value) {
    this._last = value.toLowerCase();
  }

  get lastAddress () {
    return this._last;
  }

  get (address) {
    address = address.toLowerCase();

    this.lastAddress = address;

    const account = this._store.find((account) => account.address === address);

    if (!account) {
      throw new Error(`Account not found: ${address}`);
    }

    return account;
  }

  remove (address, password) {
    address = address.toLowerCase();

    const index = this._store.findIndex((account) => account.address === address);

    if (index === -1) {
      return false;
    }

    const account = this._store[index];

    if (!account.isValidPassword(password)) {
      console.log('invalid password');
      return false;
    }

    if (address === this.lastAddress) {
      this.lastAddress = NULL_ADDRESS;
    }

    this._store.splice(index, 1);

    this.persist();

    return true;
  }

  mapArray (mapper) {
    return this._store.map(mapper);
  }

  mapObject (mapper) {
    const result = {};

    this._store.forEach((account) => {
      result[account.address] = mapper(account);
    });

    return result;
  }

  toJSON () {
    return {
      last: this._last,
      store: this._store
    };
  }
}
