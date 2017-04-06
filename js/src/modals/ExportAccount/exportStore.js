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

import { action, observable } from 'mobx';
import FileSaver from 'file-saver';

export default class ExportStore {
  @observable canExport = false;
  @observable selectedAccount = '';
  @observable selectedAccounts = {};
  @observable passwordInputs = {};

  constructor (api, accounts, newError, address) {
    this.accounts = accounts;
    this._api = api;
    this._newError = newError;
    if (address) {
      this.selectedAccounts[address] = true;
      this.selectedAccount = address;
    }
  }

  @action changePassword = (event, password) => {
    this.passwordInputs[this.selectedAccount] = password;
  }

  @action getPassword = (address) => {
    return this.passwordInputs[address];
  }

  @action onClick = (address) => {
    this.selectedAccount = address;
  }

  @action resetAccountValue = () => {
    this.passwordInputs[this.selectedAccount] = '';
  }

  @action setAccounts = (accounts) => {
    this.accounts = accounts;
  }

  @action setSelectedAccount = (addr) => {
    this.selectedAccounts[addr] = true;
    this.canExport = true;
  }

  @action toggleSelectedAccount = (addr) => {
    if (this.selectedAccounts[addr]) {
      delete this.selectedAccounts[addr];
    } else {
      this.selectedAccounts[addr] = true;
    }
    this.canExport = false;
    Object
      .keys(this.selectedAccounts)
      .forEach((address) => {
        if (this.selectedAccounts[address]) {
          this.canExport = true;
        }
      });
  }

  onExport = (event) => {
    const { parity } = this._api;
    const accounts = Object.keys(this.selectedAccounts);

    accounts.forEach((address) => {
      let password = this.passwordInputs[address];

      parity
        .exportAccount(address, password)
        .then((content) => {
          const text = JSON.stringify(content, null, 4);
          const blob = new Blob([ text ], { type: 'application/json' });
          const filename = this.accounts[address].uuid;

          FileSaver.saveAs(blob, `${filename}.json`);

          this.accountValue = '';
          if (event) { event(); }
        })
        .catch((err) => {
          const { name, meta } = this.accounts[address];
          const { passwordHint } = meta;

          this._newError({
            message: `[${err.code}] Account "${name}" - Incorrect password. (Password Hint: ${passwordHint})`
          });
        });
    });
  }
}
