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
  @observable accountValue = '';
  @observable inputValue = {};

  constructor (api, accounts, newError, address) {
    this._api = api;
    this._accounts = accounts;
    this._newError = newError;
    this._address = address;
  }

  @action changePassword = (event, password) => {
    const selectedAccount = (this._address) ? null : this.selectedAccount;

    this.setPassword(selectedAccount, password);
  }

  @action getPassword = (account) => {
    return this.inputValue[account];
  }

  @action onFocus = (address) => {
    this.selectedAccount = address;
  }

  @action resetAccountValue = () => {
    this.accountValue = '';
  }

  @action setPassword = (account, password) => {
    (this._address)
      ? this.accountValue = password
      : this.inputValue[account] = password;
  }

  @action setSelectedAccount = (addr) => {
    this.selectedAccounts[addr] = true;
    this.canExport = true;
  }

  @action toggleSelectedAccount = (addr) => {
    this.selectedAccounts[addr] = !this.selectedAccounts[addr];
    if (!this.selectedAccounts[addr]) {
      delete this.selectedAccounts[addr];
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
    const accounts = (this._address) ? [this._address] : Object.keys(this.selectedAccounts);

    accounts.forEach((account) => {
      let password = (this._address) ? this.accountValue : this.inputValue[account];

      parity
        .exportAccount(account, password)
        .then((content) => {
          const text = JSON.stringify(content, null, 4);
          const blob = new Blob([ text ], { type: 'application/json' });
          const filename = this._accounts[account].uuid;

          FileSaver.saveAs(blob, `${filename}.json`);

          this.accountValue = '';
          if (event) { event(); }
        })
        .catch((err) => {
          const { name, meta } = this._accounts[account];
          const { passwordHint } = meta;

          this._newError({
            message: `[${err.code}] Account "${name}" - Incorrect password. (Password Hint: ${passwordHint})`
          });
        });
    });
  }
}
