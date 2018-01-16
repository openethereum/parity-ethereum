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

import { personalAccountsInfo } from './personalActions';

let instance;

export default class Personal {
  constructor (store, api) {
    this._api = api;
    this._store = store;
  }

  static get (store) {
    if (!instance && store) {
      return Personal.init(store);
    }

    return instance;
  }

  static init (store) {
    const { api } = store.getState();

    if (!instance) {
      instance = new Personal(store, api);
    } else if (!instance) {
      throw new Error('The Personal Provider has not been initialized yet');
    }

    return instance;
  }

  static start () {
    const self = instance;

    return Personal.stop()
      .then(() => Promise.all([
        self._removeDeleted(),
        self._subscribeAccountsInfo()
      ]));
  }

  static stop () {
    if (!instance) {
      return Promise.resolve();
    }

    const self = instance;

    return self._unsubscribeAccountsInfo();
  }

  _subscribeAccountsInfo () {
    let resolved = false;

    // The Promise will be resolved when the first
    // accounts are loaded
    return new Promise((resolve, reject) => {
      this._api
        .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
          if (error) {
            console.error('parity_allAccountsInfo', error);

            if (!resolved) {
              resolved = true;
              return reject(error);
            }

            return;
          }

          // Add the address to each accounts
          Object.keys(accountsInfo)
            .forEach((address) => {
              accountsInfo[address].address = address;
            });

          const { dispatch, getState } = this._store;

          personalAccountsInfo(accountsInfo)(dispatch, getState)
            .then(() => {
              if (!resolved) {
                resolved = true;
                return resolve();
              }
            })
            .catch((error) => {
              if (!resolved) {
                resolved = true;
                return reject(error);
              }
            });
        })
        .then((subId) => {
          this.subscriptionId = subId;
        });
    });
  }

  _unsubscribeAccountsInfo () {
    // Unsubscribe to any previous
    // subscriptions
    if (this.subscriptionId) {
      return this._api
        .unsubscribe(this.subscriptionId)
        .then(() => {
          this.subscriptionId = null;
        });
    }

    return Promise.resolve();
  }

  _removeDeleted () {
    return this._api.parity
      .allAccountsInfo()
      .then((accountsInfo) => {
        return Promise.all(
          Object
            .keys(accountsInfo)
            .filter((address) => {
              const account = accountsInfo[address];

              return !account.uuid && account.meta && account.meta.deleted;
            })
            .map((address) => this._api.parity.removeAddress(address))
        );
      })
      .then((results) => {
        if (results.length) {
          console.log(`Removed ${results.length} previously marked addresses`);
        }
      })
      .catch((error) => {
        console.warn('removeDeleted', error);
      });
  }
}
