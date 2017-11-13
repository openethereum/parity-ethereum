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
import store from 'store';

const OLD_LS_FIRST_RUN_KEY = 'showFirstRun';
const LS_FIRST_RUN_KEY = '_parity::showFirstRun';

export default class Store {
  @observable firstrunVisible = false;

  constructor (api) {
    // Migrate the old key to the new one
    this._migrateStore();

    this._api = api;

    // Show the first run the storage doesn't hold `false` value
    const firstrunVisible = store.get(LS_FIRST_RUN_KEY) !== false;

    // Only check accounts if we might show the first run
    if (firstrunVisible) {
      api.transport.once('open', () => {
        this._checkAccounts();
      });
    } else {
      this.firstrunVisible = false;
    }
  }

  @action closeFirstrun = () => {
    this.toggleFirstrun(false);
  }

  @action toggleFirstrun = (visible = false) => {
    this.firstrunVisible = visible;

    // There's no need to write to storage that the
    // First Run should be visible
    if (!visible) {
      store.set(LS_FIRST_RUN_KEY, !!visible);
    }
  }

  /**
   * Migrate the old LocalStorage key format
   * to the new one
   */
  _migrateStore () {
    const oldValue = store.get(OLD_LS_FIRST_RUN_KEY);
    const newValue = store.get(LS_FIRST_RUN_KEY);

    if (newValue === undefined && oldValue !== undefined) {
      store.set(LS_FIRST_RUN_KEY, oldValue);
      store.remove(OLD_LS_FIRST_RUN_KEY);
    }
  }

  _checkAccounts () {
    return Promise
      .all([
        this._api.parity.listVaults(),
        this._api.parity.allAccountsInfo()
      ])
      .then(([ vaults, info ]) => {
        const accounts = Object.keys(info)
          .filter((address) => info[address].uuid)
          // In DEV mode, the empty phrase account is already added
          .filter((address) => address.toLowerCase() !== '0x00a329c0648769a73afac7f9381e08fb43dbea72');

        // Has accounts if any vaults or accounts
        const hasAccounts = (accounts && accounts.length > 0) || (vaults && vaults.length > 0);

        // Show First Run if no accounts and no vaults
        this.toggleFirstrun(!hasAccounts);
      })
      .catch((error) => {
        console.error('checkAccounts', error);
      });
  }
}
