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
import { pick, range } from 'lodash';

import Contracts from '~/contracts';
import { hashToImageUrl } from '~/redux/util';
import { bytesToHex } from '~/api/util/format';

import builtinApps from './builtin.json';

const BUILTIN_APPS_KEY = 'BUILTIN_APPS_KEY';
let dappsFetcherInstance = null;

export default class DappsFetcher {

  _manifests = {};
  _dappsUrl = '';

  _registryAppsIds = null;
  _cachedApps = {};

  constructor (api) {
    this._dappsUrl = api.dappsUrl;
  }

  static get (api) {
    if (!dappsFetcherInstance) {
      dappsFetcherInstance = new DappsFetcher(api);
    }

    return dappsFetcherInstance;
  }

  _getHost () {
    const host = process.env.DAPPS_URL || (process.env.NODE_ENV === 'production'
      ? this._dappsUrl
      : '');

    if (host === '/') {
      return '';
    }

    return host;
  }

  fetchBuiltinApps (force = false) {
    if (!force && this._cachedApps[BUILTIN_APPS_KEY]) {
      return Promise.resolve().then(() => this._cachedApps[BUILTIN_APPS_KEY]);
    }

    const { dappReg } = Contracts.get();

    this._cachedApps[BUILTIN_APPS_KEY] = Promise
      .all(builtinApps.map((app) => dappReg.getImage(app.id)))
      .then((imageIds) => {
        return builtinApps.map((app, index) => {
          app.type = 'builtin';
          app.image = hashToImageUrl(imageIds[index]);
          return app;
        });
      })
      .then((apps) => {
        this._cachedApps[BUILTIN_APPS_KEY] = apps;
        return apps;
      })
      .catch((error) => {
        console.warn('DappsStore:fetchBuiltinApps', error);
      });

    return Promise.resolve().then(() => this._cachedApps[BUILTIN_APPS_KEY]);
  }

  fetchLocalApps () {
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
      .catch((error) => {
        console.warn('DappsStore:fetchLocal', error);
      });
  }

  fetchRegistryAppIds (force = false) {
    if (!force && this._registryAppsIds) {
      return Promise.resolve().then(() => this._registryAppsIds);
    }

    const { dappReg } = Contracts.get();

    this._registryAppsIds = dappReg
      .count()
      .then((count) => {
        const promises = range(0, count.toNumber()).map((index) => dappReg.at(index));
        return Promise.all(promises);
      })
      .then((appsInfo) => {
        const appIds = appsInfo
          .map(([appId, owner]) => bytesToHex(appId))
          .filter((appId) => {
            return (new BigNumber(appId)).gt(0) && !builtinApps.find((app) => app.id === appId);
          });

        this._registryAppsIds = appIds;
        return this._registryAppsIds;
      })
      .catch((error) => {
        console.warn('DappsStore:fetchRegistryAppIds', error);
      });

    return Promise.resolve().then(() => this._registryAppsIds);
  }

  fetchRegistryApp (dappReg, appId, force = false) {
    if (!force && this._cachedApps[appId]) {
      return Promise.resolve().then(() => this._cachedApps[appId]);
    }

    this._cachedApps[appId] = Promise
      .all([
        dappReg.getImage(appId),
        dappReg.getContent(appId),
        dappReg.getManifest(appId)
      ])
      .then(([ imageId, contentId, manifestId ]) => {
        const app = {
          id: appId,
          image: hashToImageUrl(imageId),
          contentHash: bytesToHex(contentId).substr(2),
          manifestHash: bytesToHex(manifestId).substr(2),
          type: 'network',
          visible: true
        };

        return this
          ._fetchManifest(app.manifestHash)
          .then((manifest) => {
            if (manifest) {
              app.manifestHash = null;

              // Add usefull manifest fields to app
              Object.assign(app, pick(manifest, ['author', 'description', 'name', 'version']));
            }

            return app;
          });
      })
      .then((app) => {
        // Keep dapps that has a Manifest File and an Id
        const dapp = (app.manifestHash || !app.id) ? null : app;

        this._cachedApps[appId] = dapp;
        return dapp;
      })
      .catch((error) => {
        console.warn('DappsStore:fetchRegistryApp', error);
      });

    return Promise.resolve().then(() => this._cachedApps[appId]);
  }

  _fetchManifest (manifestHash) {
    if (/^(0x)?0+/.test(manifestHash)) {
      return Promise.resolve(null);
    }

    if (this._manifests[manifestHash]) {
      return Promise.resolve().then(() => this._manifests[manifestHash]);
    }

    this._manifests[manifestHash] = fetch(`${this._getHost()}/api/content/${manifestHash}/`, { redirect: 'follow', mode: 'cors' })
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

    return this._manifests[manifestHash];
  }
}
