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

const LS_SETTINGS_KEY = '_console::settings';

let instance;

export default class SettingsStore {
  @observable displayTimestamps = true;
  @observable executeOnEnter = true;

  constructor () {
    this.load();
  }

  static get () {
    if (!instance) {
      instance = new SettingsStore();
    }

    return instance;
  }

  load () {
    const settings = store.get(LS_SETTINGS_KEY) || {};
    const { executeOnEnter, displayTimestamps } = settings;

    if (executeOnEnter !== undefined) {
      this.setExecuteOnEnter(executeOnEnter);
    }

    if (displayTimestamps !== undefined) {
      this.setDisplayTimestamps(displayTimestamps);
    }
  }

  save () {
    const { executeOnEnter, displayTimestamps } = this;
    const settings = { executeOnEnter, displayTimestamps };

    store.set(LS_SETTINGS_KEY, settings);
  }

  @action
  setDisplayTimestamps (value) {
    this.displayTimestamps = value;
    this.save();
  }

  @action
  setExecuteOnEnter (value) {
    this.executeOnEnter = value;
    this.save();
  }
}
