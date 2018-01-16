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

import { action, computed, observable, transaction } from 'mobx';

let instance;

export default class AccountStore {
  @observable allAccounts = [];
  @observable defaultAccount = null;
  @observable isLoading = false;

  whitelist = []; // Whitelist of account addresses visible by dapps, i.e. parity_getNewDappsAddresses

  constructor (api) {
    this._api = api;

    this
      .subscribeDefaultAccount()
      .then(() => this.loadAccounts());
  }

  /**
   * Accounts that are whitelisted to be shown to dapps
   */
  @computed get accounts () {
    return this.allAccounts.filter(account => this.whitelist.includes(account.address));
  }

  /**
   * For backwards compatibility
   */
  @action setAccounts = (accounts) => {
    return this.setAllAccounts(accounts);
  }

  @action setAllAccounts = (allAccounts) => {
    this.allAccounts = allAccounts;
  }

  @action setDefaultAccount = (defaultAccount) => {
    transaction(() => {
      this.allAccounts = this.allAccounts.map((account) => {
        account.checked = account.address === defaultAccount;

        return account;
      });

      this.defaultAccount = defaultAccount;
    });
  }

  @action setLoading = (isLoading) => {
    this.isLoading = isLoading;
  }

  makeDefaultAccount = (defaultAddress) => {
    this.setDefaultAccount(defaultAddress);

    return this._api.parity
      .setNewDappsDefaultAddress(defaultAddress)
      .catch((error) => {
        console.warn('makeDefaultAccount', error);
      });
  }

  loadAccounts () {
    this.setLoading(true);

    return Promise
      .all([
        this._api.parity.getNewDappsAddresses(),
        this._api.parity.allAccountsInfo()
      ])
      .then(([whitelist, allAccounts]) => {
        this.whitelist = whitelist;

        transaction(() => {
          const accounts = Object
            .keys(allAccounts)
            .filter((address) => {
              const account = allAccounts[address];
              const isAccount = account.uuid;
              const isExternal = account.meta && (account.meta.external || account.meta.hardware);

              return (isAccount || isExternal);
            })
            .map((address) => {
              return {
                ...allAccounts[address],
                checked: address === this.defaultAccount,
                address
              };
            });

          this.setLoading(false);
          this.setAllAccounts(accounts);
        });
      })
      .catch((error) => {
        this.setLoading(false);
        console.warn('loadAccounts', error);
      });
  }

  subscribeDefaultAccount () {
    const promiseDefaultAccount = this._api.subscribe('parity_defaultAccount', (error, defaultAccount) => {
      if (!error) {
        this.setDefaultAccount(defaultAccount);
      }
    });

    const promiseEthAccounts = this._api.subscribe('eth_accounts', (error) => {
      if (!error) {
        this.loadAccounts();
      }
    });

    const promiseAccountsInfo = this._api
      .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
        if (!error) {
          this.loadAccounts();
        }
      });

    return Promise.all([promiseDefaultAccount, promiseEthAccounts, promiseAccountsInfo]);
  }

  static get (api) {
    if (!instance) {
      instance = new AccountStore(api);
    }

    return instance;
  }
}
