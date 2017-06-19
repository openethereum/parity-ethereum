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

import filteredRequests from '../DappRequests/filteredRequests';

export default class Store {
  @observable filteredRequests = Object.keys(filteredRequests);
  @observable modalOpen = false;
  @observable permissions = {};

  @action closeModal = () => {
    this.modalOpen = false;
  }

  @action openModal = () => {
    this.modalOpen = true;
  }

  @action addMethodPermission = (method, token) => {
    const id = `${method}:${token}`;

    this.permissions = Object.assign({}, this.permissions, {
      [id]: true
    });
  }

  @action toggleMethodPermission = (method, token) => {
    const id = `${method}:${token}`;

    this.permissions = Object.assign({}, this.permissions, {
      [id]: !this.permissions[id]
    });
  }

  static instance = null;

  static get () {
    if (!Store.instance) {
      Store.instance = new Store();
    }

    return Store.instance;
  }
}
