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
const LS_KEY_EXTERNAL_ACCEPT = 'acceptExternal';

export default class DappsStore {
  @observable apps = [];
  @observable displayApps = {};
  @observable modalOpen = false;
  @observable externalOverlayVisible = true;

  _manifests = {};

  constructor (api) {
    this._api = api;

    this.loadExternalOverlay();
    this.readDisplayApps();

    Promise
      .all([
        this._fetchBuiltinApps(),
        this._fetchLocalApps(),
        this._fetchRegistryApps()
      ])
      .then(this.writeDisplayApps);
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
    this.displayApps = Object.assign({}, this.displayApps, { [id]: { visible: false } });
    this.writeDisplayApps();
  }

  @action showApp = (id) => {
    this.displayApps = Object.assign({}, this.displayApps, { [id]: { visible: true } });
    this.writeDisplayApps();
  }

  @action readDisplayApps = () => {
    this.displayApps = store.get(LS_KEY_DISPLAY) || {};
  }

  @action writeDisplayApps = () => {
    store.set(LS_KEY_DISPLAY, this.displayApps);
  }

  @action addApps = (apps) => {
    transaction(() => {
      this.apps = this.apps
        .concat(apps || [])
        .sort((a, b) => a.name.localeCompare(b.name));

      const visibility = {};
      apps.forEach((app) => {
        if (!this.displayApps[app.id]) {
          visibility[app.id] = { visible: app.visible };
        }
      });

      this.displayApps = Object.assign({}, this.displayApps, visibility);
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
        this.addApps(
          builtinApps.map((app, index) => {
            app.type = 'builtin';
            app.image = hashToImageUrl(imageIds[index]);
            return app;
          })
        );
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
            app.visible = true;
            return app;
          })
          .filter((app) => app.id && !['ui'].includes(app.id));
      })
      .then(this.addApps)
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
                type: 'network',
                visible: true
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
      .then(this.addApps)
      .catch((error) => {
        console.warn('DappsStore:fetchRegistry', error);
      });
  }

  _fetchManifest (manifestHash) {
    if (/^(0x)?0+/.test(manifestHash)) {
      return Promise.resolve(null);
    }

    if (this._manifests[manifestHash]) {
      return Promise.resolve(this._manifests[manifestHash]);
    }

    return fetch(`${this._getHost()}/api/content/${manifestHash}/`, { redirect: 'follow', mode: 'cors' })
      .then((response) => {
        return response.ok
          ? response.json()
          : null;
      })
      .then((manifest) => {
        if (manifest) {
          this._manifests[manifestHash] = manifest;
        }

        return manifest;
      })
      .catch((error) => {
        console.warn('DappsStore:fetchManifest', error);
        return null;
      });
  }
}
