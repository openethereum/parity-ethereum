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

import { action, map, observable } from 'mobx';

import { api } from './parity';

let instance;

export default class WatchesStore {
  @observable watches = map();

  watchesFunctions = {};

  constructor () {
    api.subscribe('eth_blockNumber', () => {
      this.refreshWatches();
    });
  }

  static get () {
    if (!instance) {
      instance = new WatchesStore();
    }

    return instance;
  }

  @action
  addWatch (name, func, context) {
    if (typeof func !== 'function' && typeof func.then !== 'function') {
      throw new Error(`cannot watch ${name} ; not a Function/Promise given`);
    }

    this.watchesFunctions[name] = { func, context };
    this.watches.set(name, {});
    this.refreshWatches();
  }

  @action
  removeWatch (name) {
    this.watches.delete(name);
    delete this.watchesFunctions[name];
  }

  @action
  updateWatch (name, result, isError = false) {
    const next = {};

    if (isError) {
      next.error = result;
    } else {
      next.result = result;
    }

    this.watches.set(name, { ...next });
  }

  refreshWatches () {
    const names = this.watches.keys();
    const promises = names
      .map((name) => {
        const { context, func } = this.watchesFunctions[name];
        let result;

        try {
          if (typeof func === 'function') {
            result = func.apply(context || this);
          } else {
            result = func;
          }

          return Promise.resolve(result);
        } catch (error) {
          return Promise.reject(error);
        }
      })
      .map((promise, index) => {
        const name = names[index];

        return promise
          .then((result) => {
            this.updateWatch(name, result);
          })
          .catch((error) => {
            this.updateWatch(name, error, true);
          });
      });

    return Promise.all(promises);
  }
}
