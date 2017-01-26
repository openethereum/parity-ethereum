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

import EventEmitter from 'eventemitter3';
import { action, computed, observable, transaction } from 'mobx';
import store from 'store';

import Contracts from '~/contracts';
import {
  fetchBuiltinApps, fetchLocalApps,
  fetchRegistryAppIds, fetchRegistryApp,
  subscribeToChanges
} from '~/util/dapps';

const LS_KEY_DISPLAY = 'displayApps';
const LS_KEY_EXTERNAL_ACCEPT = 'acceptExternal';
const BUILTIN_APPS_KEY = 'BUILTIN_APPS_KEY';

let instance = null;

export default class DappsStore extends EventEmitter {
  @observable apps = [];
  @observable displayApps = {};
  @observable modalOpen = false;
  @observable externalOverlayVisible = true;

  _api = null;
  _subscriptions = {};
  _cachedApps = {};
  _manifests = {};
  _registryAppsIds = null;

  constructor (api) {
    super();

    this._api = api;

    this.readDisplayApps();
    this.loadExternalOverlay();
    this.subscribeToChanges();
  }

  static get (api) {
    if (!instance) {
      instance = new DappsStore(api);
    }

    return instance;
  }

  /**
   * Try to find the app from the local (local or builtin)
   * apps, else fetch from the node
   */
  loadApp (id) {
    const { dappReg } = Contracts.get();

    return this
      .loadLocalApps()
      .then(() => {
        const app = this.apps.find((app) => app.id === id);

        if (app) {
          return app;
        }

        return this.fetchRegistryApp(dappReg, id, true);
      })
      .then((app) => {
        this.emit('loaded', app);
        return app;
      });
  }

  loadLocalApps () {
    return Promise
      .all([
        this.fetchBuiltinApps().then((apps) => this.addApps(apps)),
        this.fetchLocalApps().then((apps) => this.addApps(apps))
      ]);
  }

  loadAllApps () {
    const { dappReg } = Contracts.get();

    return Promise
      .all([
        this.loadLocalApps(),
        this.fetchRegistryApps(dappReg).then((apps) => this.addApps(apps))
      ])
      .then(this.writeDisplayApps);
  }

  subscribeToChanges () {
    const { dappReg } = Contracts.get();

    // Unsubscribe from previous subscriptions, if any
    if (this._subscriptions.block) {
      this._api.unsubscribe(this._subscriptions.block);
    }

    if (this._subscriptions.filter) {
      this._api.eth.uninstallFilter(this._subscriptions.filter);
    }

    // Subscribe to dapps reg changes
    subscribeToChanges(this._api, dappReg, (appIds) => {
      const updates = appIds.map((appId) => {
        return this.fetchRegistryApp(dappReg, appId, true);
      });

      Promise
        .all(updates)
        .then((apps) => {
          this.addApps(apps);
        });
    }).then((subscriptions) => {
      this._subscriptions = subscriptions;
    });
  }

  fetchBuiltinApps (force = false) {
    if (!force && this._cachedApps[BUILTIN_APPS_KEY] !== undefined) {
      return Promise.resolve(this._cachedApps[BUILTIN_APPS_KEY]);
    }

    this._cachedApps[BUILTIN_APPS_KEY] = fetchBuiltinApps()
      .then((apps) => {
        this._cachedApps[BUILTIN_APPS_KEY] = apps;
        return apps;
      });

    return Promise.resolve(this._cachedApps[BUILTIN_APPS_KEY]);
  }

  fetchLocalApps () {
    return fetchLocalApps(this._api);
  }

  fetchRegistryAppIds (force = false) {
    if (!force && this._registryAppsIds) {
      return Promise.resolve(this._registryAppsIds);
    }

    this._registryAppsIds = fetchRegistryAppIds()
      .then((appIds) => {
        this._registryAppsIds = appIds;
        return this._registryAppsIds;
      });

    return Promise.resolve(this._registryAppsIds);
  }

  fetchRegistryApp (dappReg, appId, force = false) {
    if (!force && this._cachedApps[appId] !== undefined) {
      return Promise.resolve(this._cachedApps[appId]);
    }

    this._cachedApps[appId] = fetchRegistryApp(this._api, dappReg, appId)
      .then((dapp) => {
        this._cachedApps[appId] = dapp;
        return dapp;
      });

    return Promise.resolve(this._cachedApps[appId]);
  }

  fetchRegistryApps (dappReg) {
    return this
      .fetchRegistryAppIds()
      .then((appIds) => {
        const promises = appIds.map((appId) => {
          // Fetch the Dapp and display it ASAP
          return this
            .fetchRegistryApp(dappReg, appId)
            .then((app) => {
              if (app) {
                this.addApps([ app ]);
              }

              return app;
            });
        });

        return Promise.all(promises);
      });
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
    return this.visibleApps.filter((app) => app.type === 'builtin');
  }

  @computed get visibleLocal () {
    return this.visibleApps.filter((app) => app.type === 'local');
  }

  @computed get visibleNetwork () {
    return this.visibleApps.filter((app) => app.type === 'network');
  }

  @action openModal = () => {
    this.modalOpen = true;
  }

  @action closeModal = () => {
    this.modalOpen = false;
  }

  @action closeExternalOverlay = () => {
    this.externalOverlayVisible = false;
    store.set(LS_KEY_EXTERNAL_ACCEPT, true);
  }

  @action loadExternalOverlay () {
    this.externalOverlayVisible = !(store.get(LS_KEY_EXTERNAL_ACCEPT) || false);
  }

  @action hideApp = (id) => {
    this.setDisplayApps({ [id]: { visible: false } });
    this.writeDisplayApps();
  }

  @action showApp = (id) => {
    this.setDisplayApps({ [id]: { visible: true } });
    this.writeDisplayApps();
  }

  @action readDisplayApps = () => {
    this.displayApps = store.get(LS_KEY_DISPLAY) || {};
  }

  @action writeDisplayApps = () => {
    store.set(LS_KEY_DISPLAY, this.displayApps);
  }

  @action setDisplayApps = (displayApps) => {
    this.displayApps = Object.assign({}, this.displayApps, displayApps);
  };

  @action addApps = (_apps = []) => {
    transaction(() => {
      const apps = _apps.filter((app) => app);

      // Get new apps IDs if available
      const newAppsIds = apps
        .map((app) => app.id)
        .filter((id) => id);

      this.apps = this.apps
        .filter((app) => !app.id || !newAppsIds.includes(app.id))
        .concat(apps || [])
        .sort((a, b) => a.name.localeCompare(b.name));

      const visibility = {};

      apps.forEach((app) => {
        if (!this.displayApps[app.id]) {
          visibility[app.id] = { visible: app.visible };
        }
      });

      this.setDisplayApps(visibility);
    });
  }
}

export {
  LS_KEY_DISPLAY
};
