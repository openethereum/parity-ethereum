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

import BigNumber from 'bignumber.js';
import { action, computed, observable, transaction } from 'mobx';
import store from 'store';

import Contracts from '../../contracts';
import { hashToImageUrl } from '../../redux/util';

import builtinApps from './builtin.json';

const LS_KEY_DISPLAY = 'displayApps';

export default class DappsStore {
  @observable apps = [];
  @observable displayApps = {};
  @observable modalOpen = false;

  constructor (api) {
    this._api = api;

    this._readDisplayApps();

    Promise
      .all([
        this._fetchBuiltinApps(),
        this._fetchLocalApps(),
        this._fetchRegistryApps()
      ])
      .then(this._writeDisplayApps);
  }

  @computed get sortedApps () {
    return this.apps.sort((a, b) => a.name.localeCompare(b.name));
  }

  @computed get visibleApps () {
    return this.sortedApps.filter((app) => this.displayApps[app.id] && this.displayApps[app.id].visible);
  }

  @action openModal = () => {
    this.modalOpen = true;
  }

  @action closeModal = () => {
    this.modalOpen = false;
  }

  @action hideApp = (id) => {
    const app = this.displayApps[id] || {};
    app.visible = false;

    this.displayApps[id] = app;
    this._writeDisplayApps();
  }

  @action showApp = (id) => {
    const app = this.displayApps[id] || {};
    app.visible = true;

    this.displayApps[id] = app;
    this._writeDisplayApps();
  }

  @action setDisplayApps = (apps) => {
    this.displayApps = apps;
  }

  @action addApp = (app, visible) => {
    transaction(() => {
      this.apps.push(app);

      if (!this.displayApps[app.id]) {
        this.displayApps[app.id] = { visible };
      }
    });
  }

  _getHost (api) {
    return process.env.NODE_ENV === 'production'
      ? this._api.dappsUrl
      : '';
  }

  _fetchBuiltinApps () {
    const { dappReg } = Contracts.get();

    return Promise
      .all(builtinApps.map((app) => dappReg.getImage(app.id)))
      .then((imageIds) => {
        transaction(() => {
          builtinApps.forEach((app, index) => {
            app.type = 'builtin';
            app.image = hashToImageUrl(imageIds[index]);
            this.addApp(app, app.visible);
          });
        });
      })
      .catch((error) => {
        console.warn('DappsStore:fetchBuiltinApps', error);
      });
  }

  _fetchLocalApps () {
    return fetch(`${this._getHost()}/api/apps`)
      .then((response) => {
        return response.ok
          ? response.json()
          : [];
      })
      .then((apps) => {
        return apps
          .map((app) => {
            app.type = 'local';
            return app;
          })
          .filter((app) => app.id && !['ui'].includes(app.id));
      })
      .then((apps) => {
        transaction(() => {
          (apps || []).forEach((app) => this.addApp(app, true));
        });
      })
      .catch((error) => {
        console.warn('DappsStore:fetchLocal', error);
      });
  }

  _fetchRegistryApps () {
    const { dappReg } = Contracts.get();

    return dappReg
      .count()
      .then((_count) => {
        const count = _count.toNumber();
        const promises = [];

        for (let index = 0; index < count; index++) {
          promises.push(dappReg.at(index));
        }

        return Promise.all(promises);
      })
      .then((appsInfo) => {
        const appIds = appsInfo
          .map(([appId, owner]) => this._api.util.bytesToHex(appId))
          .filter((appId) => {
            return (new BigNumber(appId)).gt(0) && !builtinApps.find((app) => app.id === appId);
          });

        return Promise
          .all([
            Promise.all(appIds.map((appId) => dappReg.getImage(appId))),
            Promise.all(appIds.map((appId) => dappReg.getContent(appId))),
            Promise.all(appIds.map((appId) => dappReg.getManifest(appId)))
          ])
          .then(([imageIds, contentIds, manifestIds]) => {
            return appIds.map((appId, index) => {
              const app = {
                id: appId,
                image: hashToImageUrl(imageIds[index]),
                contentHash: this._api.util.bytesToHex(contentIds[index]).substr(2),
                manifestHash: this._api.util.bytesToHex(manifestIds[index]).substr(2),
                type: 'network'
              };

              return app;
            });
          });
      })
      .then((apps) => {
        return Promise
          .all(apps.map((app) => this._fetchManifest(app.manifestHash)))
          .then((manifests) => {
            return apps.map((app, index) => {
              const manifest = manifests[index];

              if (manifest) {
                app.manifestHash = null;
                Object.keys(manifest)
                  .filter((key) => ['author', 'description', 'name', 'version'].includes(key))
                  .forEach((key) => {
                    app[key] = manifest[key];
                  });
              }

              return app;
            });
          })
          .then((apps) => {
            return apps.filter((app) => {
              return !app.manifestHash && app.id;
            });
          });
      })
      .then((apps) => {
        transaction(() => {
          (apps || []).forEach((app) => this.addApp(app, false));
        });
      })
      .catch((error) => {
        console.warn('DappsStore:fetchRegistry', error);
      });
  }

  _fetchManifest (manifestHash) {
    return fetch(`${this._getHost()}/api/content/${manifestHash}/`, { redirect: 'follow', mode: 'cors' })
      .then((response) => {
        return response.ok
          ? response.json()
          : null;
      })
      .catch((error) => {
        console.warn('DappsStore:fetchManifest', error);
        return null;
      });
  }

  _readDisplayApps = () => {
    this.setDisplayApps(store.get(LS_KEY_DISPLAY) || {});
  }

  _writeDisplayApps = () => {
    store.set(LS_KEY_DISPLAY, this.displayApps);
  }
}
