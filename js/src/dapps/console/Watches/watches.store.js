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

import { api } from '../parity';
import { evaluate } from '../utils';

let instance;

export default class WatchesStore {
  @observable showAdd = false;
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

      window.watch = instance.add.bind(instance);
      window.unwatch = instance.remove.bind(instance);
    }

    return instance;
  }

  @action
  add (name, func, context) {
    if (!func || (typeof func !== 'function' && typeof func.then !== 'function')) {
      return console.error(Error(`cannot watch ${name} ; not a Function/Promise given`));
    }

    this.watchesFunctions[name] = { func, context };
    this.watches.set(name, {});
    this.refreshWatches();
  }

  addWatch () {
    this.toggleAdd();

    const { addContext, addFunction, addName } = this;

    const evaluatedFunction = evaluate(addFunction);
    const evaluatedContext = addContext
      ? evaluate(addContext)
      : {};

    this.add(addName, evaluatedFunction.result, evaluatedContext.result);
  }

  get (name) {
    return this.watches.get(name);
  }

  get names () {
    return this.watches.keys();
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

  @action
  remove (name) {
    this.watches.delete(name);
    delete this.watchesFunctions[name];
  }

  @action
  toggleAdd () {
    this.showAdd = !this.showAdd;
  }

  updateAddContext (value) {
    this.addContext = value;
  }

  updateAddFunction (value) {
    this.addFunction = value;
  }

  updateAddName (value) {
    this.addName = value;
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
}
