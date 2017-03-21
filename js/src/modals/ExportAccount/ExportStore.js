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

class ExportStore {
  @observable canExport = false;
  @observable selectedAccounts = {};
  @observable inputValue = {};

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

  @actions getPassword = (account) => {
    return this.inputValue[account];
  }

  @actions setPassword = (account, password) => {
    this.inputValue[account.address] = password;
  }
}

export default new ExportStore();
