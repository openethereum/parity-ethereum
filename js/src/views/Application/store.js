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

export default class Store {
  @observable firstrunVisible = false;

  constructor (api) {
    this._api = api;
    this.firstrunVisible = store.get('showFirstRun');

    this._checkAccounts();
  }

  @action closeFirstrun = () => {
    this.toggleFirstrun(false);
  }

  @action toggleFirstrun = (visible = false) => {
    this.firstrunVisible = visible;
    store.set('showFirstRun', !!visible);
  }

  _checkAccounts () {
    this._api.parity
      .allAccountsInfo()
      .then((info) => {
        const accounts = Object.keys(info).filter((address) => info[address].uuid);

        this.toggleFirstrun(this.firstrunVisible || !accounts || !accounts.length);
      })
      .catch((error) => {
        console.error('checkAccounts', error);
      });
  }
}
