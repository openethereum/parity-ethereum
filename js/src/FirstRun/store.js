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

const LS_FIRST_RUN_KEY = '_parity::showFirstRun';

export default class Store {
  @observable visible = false;

  constructor (api) {
    this.toggle(store.get(LS_FIRST_RUN_KEY) !== false);
  }

  @action close = () => {
    this.toggle(false);
  }

  @action toggle = (visible = false) => {
    this.visible = visible;

    // There's no need to write to storage that the
    // First Run should be visible
    if (!visible) {
      store.set(LS_FIRST_RUN_KEY, !!visible);
    }
  }
}
