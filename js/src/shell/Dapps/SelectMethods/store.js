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

export default class Store {
  @observable isModalOpen = false;
  @observable methods = [];
  @observable permissions = {};

  constructor (api) {
    this._api = api;

    this.loadPermissions();
  }

  @action closeModal = () => {
    this.isModalOpen = false;

    this.savePermissions();
  }

  @action openModal = () => {
    this.isModalOpen = true;
  }

  @action setMethods = (methods) => {
    this.methods = methods;
  }

  @action setPermissions = (permissions) => {
    this.permissions = permissions;
  }

  @action toggleAppPermission = (method, appId) => {
    const id = `${method}:${appId}`;

    this.permissions = Object.assign({}, this.permissions, {
      [id]: !this.permissions[id]
    });
  }

  hasAppPermission = (method, appId) => {
    return this.permissions[`${method}:${appId}`] || false;
  }

  loadPermissions = () => {
    return Promise
      .all([
        this._api.shell.getMethodPermissions(),
        this._api.shell.getFilteredMethods()
      ])
      .then(([permissions, methods]) => {
        this.setPermissions(permissions);
        this.setMethods(methods);
      });
  }

  savePermissions = () => {
    this._api.shell.setMethodPermissions(this.permissions);
  }
}
