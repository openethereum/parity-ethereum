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
import localStore from 'store';

const LS_KEY = '_parity::history';
const MAX_ENTRIES = 5;

const instances = {};

export default class Store {
  @observable history = [];

  constructor (type) {
    this.historyKey = `${LS_KEY}::${type}`;
    this.load();
  }

  @action add = (entry, type) => {
    this.history = [{
      entry,
      timestamp: Date.now(),
      type
    }].concat(this.history.filter((h) => h.entry !== entry)).slice(0, MAX_ENTRIES);
    this.save();
  }

  @action clear = () => {
    this.history = [];
    this.save();
  }

  @action load = () => {
    this.history = localStore.get(this.historyKey) || [];
  }

  save = () => {
    return localStore.set(this.historyKey, this.history);
  }

  static get (type) {
    if (!instances[type]) {
      instances[type] = new Store(type);
    }

    return instances[type];
  }
}
