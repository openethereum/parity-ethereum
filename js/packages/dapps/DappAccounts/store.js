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

import { action, observable, transaction } from 'mobx';

export default class Store {
  @observable accounts = [];
  @observable modalOpen = false;
  @observable whitelist = [];
  @observable whitelistDefault = null;

  constructor (api) {
    this._api = api;

    this.load();
  }

  save = () => {
    const checkedAccounts = this.accounts.filter((account) => account.checked);
    const defaultAddress = (this.accounts.find((account) => account.default) || {}).address;
    const addresses = checkedAccounts.length === this.accounts.length
      ? null
      : checkedAccounts.map((account) => account.address);

    this.updateWhitelist(addresses, defaultAddress);
  }

  // FIXME: Hardware accounts are not showing up here
  @action setAccounts = (accounts) => {
    transaction(() => {
      this.accounts = Object
        .keys(accounts)
        .filter((address) => {
          const account = accounts[address];

          if (account.uuid) {
            return true;
          } else if (account.meta.hardware) {
            account.hardware = true;
            return true;
          } else if (account.meta.external) {
            account.external = true;
            return true;
          }

          return false;
        })
        .map((address, index) => {
          const account = accounts[address];

          return {
            address,
            checked: this.whitelist
              ? this.whitelist.includes(address)
              : true,
            default: this.whitelistDefault
              ? this.whitelistDefault === address
              : index === 0,
            description: account.meta.description,
            name: account.name
          };
        });
    });
  }

  @action selectAccount = (address) => {
    transaction(() => {
      const isSingleAccount = this.accounts.filter((account) => account.checked).length === 1;

      this.accounts = this.accounts.map((account) => {
        if (account.address === address && (!isSingleAccount || !account.checked)) {
          account.checked = !account.checked;
          account.default = false;
        }

        return account;
      });

      this.setDefaultAccount((
        this.accounts.find((account) => account.default) ||
        this.accounts.find((account) => account.checked) ||
        {}
      ).address);
    });
  }

  @action setDefaultAccount = (address) => {
    this.accounts = this.accounts.map((account) => {
      if (account.address === address) {
        account.checked = true;
        account.default = true;
      } else if (account.default) {
        account.default = false;
      }

      return account;
    });
  }

  @action setWhitelist = (whitelist, whitelistDefault) => {
    transaction(() => {
      this.whitelist = whitelist;
      this.whitelistDefault = whitelistDefault;
    });
  }

  load () {
    return Promise
      .all([
        this._api.parity.allAccountsInfo(),
        this._api.parity.getNewDappsAddresses(),
        this._api.parity.getNewDappsDefaultAddress()
      ])
      .then(([accounts, whitelist, whitelistDefault]) => {
        this.setWhitelist(whitelist, whitelistDefault);
        this.setAccounts(accounts);
      })
      .catch((error) => {
        console.warn('load', error);
      });
  }

  updateWhitelist (whitelist, whitelistDefault = null) {
    return Promise
      .all([
        this._api.parity.setNewDappsAddresses(whitelist),
        this._api.parity.setNewDappsDefaultAddress(whitelistDefault)
      ])
      .then(() => {
        this.setWhitelist(whitelist, whitelistDefault);
      })
      .catch((error) => {
        console.warn('updateWhitelist', error);
      });
  }
}
