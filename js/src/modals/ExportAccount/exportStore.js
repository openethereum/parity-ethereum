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
  @observable selectedAccounts = {};
  @observable accountValue = '';
  @observable inputValue = {};

  constructor (api, accounts, newError, address) {
    this._api = api;
    this._accounts = accounts;
    this._newError = newError;
    this._address = address;
  }

  @action toggleSelectedAccount = (addr) => {
    this.selectedAccounts[addr] = true;
    this.canExport = false;
    Object
      .keys(this.selectedAccounts)
      .forEach((address) => {
        if (addr !== address) {
          this.selectedAccounts[address] = false;
        }
        if (this.selectedAccounts[address]) {
          this.canExport = true;
        }
      });
  }

  @action getPassword = (account) => {
    return this.inputValue[account];
  }

  @action setPassword = (account, password) => {
    (this._address)
      ? this.accountValue = password
      : this.inputValue[account] = password;
  }

  @action changePassword = (event, password) => {
    const selectedAccount = (this._address) ? null : this.getSelectedAccount();

    this.setPassword(selectedAccount, password);
  }

  onExport = () => {
    const { parity } = this._api;
    const account = (this._address) ? this._address : this.getSelectedAccount();
    const password = (this._address) ? this.accountValue : this.inputValue[account];

    parity.exportAccount(account, password)
      .then((content) => {
        const text = JSON.stringify(content, null, 4);
        const blob = new Blob([ text ], { type: 'application/json' });
        const filename = this._accounts[account].uuid;

        FileSaver.saveAs(blob, `${filename}.json`);
      })
      .catch((err) => {
        const { passwordHint } = this._accounts[account].meta;

        this._newError({
          message: `[${err.code}] - Incorrect password. Password Hint: (${passwordHint})`
        });
      });
  }

  getSelectedAccount () {
    return Object
      .keys(this.selectedAccounts)
      .filter((account) => this.selectedAccounts[account])[0];
  }
}
