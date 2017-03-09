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

export default class Personal {
  constructor (store, api) {
    this._api = api;
    this._store = store;
  }

  start () {
    this._removeDeleted();
    this._subscribeAccountsInfo();
  }

  _subscribeAccountsInfo () {
    this._api
      .subscribe('parity_allAccountsInfo', (error, accountsInfo) => {
        if (error) {
          console.error('parity_allAccountsInfo', error);
          return;
        }

        this._store.dispatch(personalAccountsInfo(accountsInfo));
      });
  }

  _removeDeleted () {
    this._api.parity
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
