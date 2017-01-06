// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

const DEFAULT_MODE = 'active';

export default class Store {
  @observable mode = DEFAULT_MODE;

  constructor (api) {
    this._api = api;
  }

  @action setMode = (mode) => {
    this.mode = mode;
  }

  changeMode (mode) {
    return this._api.parity
      .setMode(mode)
      .then((result) => {
        if (result) {
          this.setMode(mode);
        }
      })
      .catch((error) => {
        console.warn('changeMode', error);
      });
  }

  loadMode () {
    return this._api.parity
      .mode()
      .then((mode) => {
        this.setMode(mode);
      })
      .catch((error) => {
        console.warn('loadMode', error);
      });
  }
}
