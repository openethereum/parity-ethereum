// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import DappsStore from './dappsStore';

let instance = null;

export default class ModalStore {
  @observable showingDelete = false;
  @observable showingRegister = false;
  @observable showingUpdate = false;

  _dappsStore = DappsStore.instance();

  static instance () {
    if (!instance) {
      instance = new ModalStore();
    }

    return instance;
  }

  @action showDelete () {
    this.showingDelete = true;
  }

  @action hideDelete () {
    this.showingDelete = false;
  }

  @action showRegister () {
    this.showingRegister = true;
  }

  @action hideRegister () {
    this.showingRegister = false;
  }

  @action showUpdate () {
    this.showingUpdate = true;
  }

  @action hideUpdate () {
    this.showingUpdate = false;
  }
}
