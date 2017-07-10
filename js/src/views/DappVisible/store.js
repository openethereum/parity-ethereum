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

import { action, computed, observable } from 'mobx';

export default class DappsStore {
  @observable apps = [];
  @observable displayApps = {};

  _api = null;

  constructor (api) {
    this._api = api;

    this.loadApps();
  }

  @computed get sortedBuiltin () {
    return this.apps.filter((app) => app.type === 'builtin');
  }

  @computed get sortedLocal () {
    return this.apps.filter((app) => app.type === 'local');
  }

  @computed get sortedNetwork () {
    return this.apps.filter((app) => app.type === 'network');
  }

  @computed get visibleApps () {
    return this.apps.filter((app) => this.displayApps[app.id] && this.displayApps[app.id].visible);
  }

  @computed get visibleBuiltin () {
    return this.visibleApps.filter((app) => !app.noselect && app.type === 'builtin');
  }

  @computed get visibleLocal () {
    return this.visibleApps.filter((app) => app.type === 'local');
  }

  @computed get visibleNetwork () {
    return this.visibleApps.filter((app) => app.type === 'network');
  }

  @computed get visibleViews () {
    return this.visibleApps.filter((app) => !app.noselect && app.type === 'view');
  }

  @action setApps = (apps) => {
    this.apps = apps;
  }

  @action setDisplayApps = (displayApps) => {
    this.displayApps = Object.assign({}, this.displayApps, displayApps);
  };

  @action hideApp = (id) => {
    this.setDisplayApps({ [id]: { visible: false } });
    this._api.shell.setAppVisibility(id, false);
  }

  @action showApp = (id) => {
    this.setDisplayApps({ [id]: { visible: true } });
    this._api.shell.setAppVisibility(id, true);
  }

  getAppById = (id) => {
    return this.apps.find((app) => app.id === id);
  }

  loadApps () {
    return Promise
      .all([
        this._api.shell.getApps(true),
        this._api.shell.getApps(false)
      ])
      .then(([all, displayed]) => {
        this.setDisplayApps(
          displayed.reduce((result, { id }) => {
            result[id] = { visible: true };
            return result;
          }, {})
        );
        this.setApps(all);
      });
  }
}
