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

export default class AccountStore {
  @observable accounts = [];
  @observable defaultAccount = null;
  @observable isLoading = false;

  constructor (api) {
    this._api = api;

    this.loadDefaultAccount()
      .then(() => this.loadAccounts());

    this.subscribeDefaultAccount();
  }

  @action setAccounts = (accounts) => {
    this.accounts = accounts;
  }

  @action setDefaultAccount = (defaultAccount) => {
    transaction(() => {
      this.accounts = this.accounts.map((account) => {
        account.default = account.address === defaultAccount;

        return account;
      });

      this.defaultAccount = defaultAccount;
    });
  }

  @action setLoading = (isLoading) => {
    this.isLoading = isLoading;
  }

  makeDefaultAccount = (address) => {
    const accounts = [address].concat(
      this.accounts
        .filter((account) => account.address !== address)
        .map((account) => account.address)
    );

    // Have optimistic UI: https://www.smashingmagazine.com/2016/11/true-lies-of-optimistic-user-interfaces/?utm_source=codropscollective
    this.setDefaultAccount(address);

    return this._api.parity
      .setNewDappsWhitelist(accounts)
      .catch((error) => {
        console.warn('makeDefaultAccount', error);
      });
  }

  loadDefaultAccount () {
    return this._api.parity
      .defaultAccount()
      .then((address) => this.setDefaultAccount(address));
  }

  loadAccounts () {
    this.setLoading(true);

    return Promise
      .all([
        this._api.parity.getNewDappsWhitelist(),
        this._api.parity.allAccountsInfo()
      ])
      .then(([whitelist, accounts]) => {
        transaction(() => {
          this.setLoading(false);
          this.setAccounts(
            Object
              .keys(accounts)
              .filter((address) => {
                const isAccount = accounts[address].uuid;
                const isWhitelisted = !whitelist || whitelist.includes(address);

                return isAccount && isWhitelisted;
              })
              .map((address) => {
                const account = accounts[address];

                account.address = address;
                account.default = address === this.defaultAccount;

                return account;
              })
          );
        });
      })
      .catch((error) => {
        this.setLoading(false);
        console.warn('loadAccounts', error);
      });
  }

  subscribeDefaultAccount () {
    return this._api.subscribe('parity_defaultAccount', (error, defaultAccount) => {
      if (!error) {
        this.setDefaultAccount(defaultAccount);
      }
    });
  }
}
