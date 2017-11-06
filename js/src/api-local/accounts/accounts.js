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

const EventEmitter = require('eventemitter3');
const { debounce } = require('lodash');
const localStore = require('store');

const Account = require('./account');
const { decryptPrivateKey } = require('../ethkey');

const NULL_ADDRESS = '0x0000000000000000000000000000000000000000';
const LS_STORE_KEY = '_parity::localAccounts';

class Accounts extends EventEmitter {
  constructor (data = localStore.get(LS_STORE_KEY) || {}) {
    super();

    this.persist = debounce(() => {
      this._lastState = JSON.stringify(this);

      localStore.set(LS_STORE_KEY, this);
    }, 100);

    this._addAccount = this._addAccount.bind(this);
    this._lastState = JSON.stringify(data);

    window.addEventListener('storage', ({ key, newValue }) => {
      if (key !== LS_STORE_KEY) {
        return;
      }

      if (newValue !== this._lastState) {
        console.log('Data changed in a second tab, syncing state');

        this.restore(JSON.parse(newValue));
      }
    });

    this.restore(data);
  }

  restore (data) {
    const {
      last = NULL_ADDRESS,
      dappsDefault = NULL_ADDRESS,
      store = {}
    } = data;

    this._last = last;
    this._dappsDefaultAddress = dappsDefault;
    this._store = {};

    if (Array.isArray(store)) {
      // Recover older version that stored accounts as an array
      store.forEach((data) => {
        const account = new Account(this.persist, data);

        this._store[account.address] = account;
      });
    } else {
      Object.keys(store).forEach((key) => {
        this._store[key] = new Account(this.persist, store[key]);
      });
    }
  }

  _addAccount (account) {
    const { address } = account;

    if (address in this._store && this._store[address].uuid) {
      throw new Error(`Account ${address} already exists!`);
    }

    this._store[address] = account;
    this.lastAddress = address;

    this.persist();

    return account.address;
  }

  create (secret, password) {
    const privateKey = Buffer.from(secret.slice(2), 'hex');

    return Account
      .fromPrivateKey(this.persist, privateKey, password)
      .then(this._addAccount);
  }

  restoreFromWallet (wallet, password) {
    return decryptPrivateKey(wallet, password)
      .then((privateKey) => {
        if (!privateKey) {
          throw new Error('Invalid password');
        }

        return Account.fromPrivateKey(this.persist, privateKey, password);
      })
      .then(this._addAccount);
  }

  set lastAddress (value) {
    this._last = value.toLowerCase();
  }

  get lastAddress () {
    return this._last;
  }

  get dappsDefaultAddress () {
    if (this._dappsDefaultAddress === NULL_ADDRESS) {
      return this._last;
    }

    if (this._dappsDefaultAddress in this._store) {
      return this._dappsDefaultAddress;
    }

    return NULL_ADDRESS;
  }

  set dappsDefaultAddress (value) {
    this._dappsDefaultAddress = value.toLowerCase();

    this.emit('dappsDefaultAddressChange', this._dappsDefaultAddress);

    this.persist();
  }

  get (address) {
    address = address.toLowerCase();

    const account = this._store[address];

    if (!account) {
      throw new Error(`Account not found: ${address}`);
    }

    this.lastAddress = address;

    return account;
  }

  getLazyCreate (address) {
    address = address.toLowerCase();

    this.lastAddress = address;

    if (!(address in this._store)) {
      this._store[address] = new Account(this.persist);
    }

    return this._store[address];
  }

  remove (address, password) {
    address = address.toLowerCase();

    const account = this.get(address);

    if (!account) {
      return false;
    }

    if (!account.uuid) {
      this.removeUnsafe(address);

      return true;
    }

    return account
      .isValidPassword(password)
      .then((isValid) => {
        if (!isValid) {
          return false;
        }

        if (address === this.lastAddress) {
          this.lastAddress = NULL_ADDRESS;
        }

        this.removeUnsafe(address);

        return true;
      });
  }

  removeUnsafe (address) {
    address = address.toLowerCase();

    delete this._store[address];

    this.persist();
  }

  allAddresses () {
    return Object.keys(this._store);
  }

  accountAddresses () {
    return Object
      .keys(this._store)
      .filter((address) => this._store[address].uuid);
  }

  map (mapper) {
    const result = {};

    Object.keys(this._store).forEach((key) => {
      result[key] = mapper(this._store[key]);
    });

    return result;
  }

  toJSON () {
    return {
      last: this._last,
      dappsDefault: this._dappsDefaultAddress,
      store: this._store
    };
  }
}

module.exports = Accounts;
