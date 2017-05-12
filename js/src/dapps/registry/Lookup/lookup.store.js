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

import ApplicationStore from '../Application/application.store';
import EntryStore from '../Entry/entry.store';

import { getInfo, isOwned } from '../util/registry';
import { postTransaction } from '../util/transactions';

let instance;

export default class LookupStore {
  @observable loading = false;
  @observable result = null;
  @observable reserving = null;

  applicationStore = ApplicationStore.get();

  static get () {
    if (!instance) {
      instance = new LookupStore();
    }

    return instance;
  }

  lookup (name) {
    const { contract } = this.applicationStore;

    // Show loading if request takes more than
    // 50ms
    const loadingTimeout = setTimeout(() => {
      this.setLoading(true);
    }, 50);

    return isOwned(contract, name)
      .then((reserved) => {
        if (!reserved) {
          return { name, free: true };
        }

        return getInfo(contract, name)
          .then((data) => {
            return new EntryStore(data);
          });
      })
      .then((result) => {
        this.setResult(result);
      })
      .catch((error) => {
        console.error(`could not lookup ${name}`, error);
        this.setResult(null);
      })
      .then(() => {
        clearTimeout(loadingTimeout);
        this.setLoading(false);
      });
  }

  refresh () {
    if (!this.result || !this.result.name) {
      return;
    }

    return this.lookup(this.result.name);
  }

  register () {
    if (!this.result || !this.result.name) {
      return;
    }

    const { api, contract } = this.applicationStore;
    const { name } = this.result;
    const hash = api.util.sha3.text(name.toLowerCase());

    const options = { value: this.applicationStore.fee };
    const values = [ hash ];

    this.setReserving(name);
    return postTransaction(api, contract.instance.reserve, options, values)
      .then(() => {
        this.lookup(name);
      })
      .catch((error) => {
        console.error(`reserving ${name}`, error);
      })
      .then(() => {
        this.setReserving(null);
      });
  }

  @action
  setLoading (value) {
    this.loading = value;
  }

  @action
  setResult (result) {
    this.result = result;
  }

  @action
  setReserving (reserving) {
    this.reserving = reserving;
  }
}
