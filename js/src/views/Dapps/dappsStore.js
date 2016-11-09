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

import { action, computed, observable } from 'mobx';

import Contracts from '../../contracts';
import { hashToImageUrl } from '../../redux/util';

const builtinApps = [
  {
    id: '0xf9f2d620c2e08f83e45555247146c62185e4ab7cf82a4b9002a265a0d020348f',
    url: 'basiccoin',
    name: 'Token Deployment',
    description: 'Deploy new basic tokens that you are able to send around',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0xd1adaede68d344519025e2ff574650cd99d3830fe6d274c7a7843cdc00e17938',
    url: 'registry',
    name: 'Registry',
    description: 'A global registry of addresses on the network',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0x0a8048117e51e964628d0f2d26342b3cd915248b59bcce2721e1d05f5cfa2208',
    url: 'tokenreg',
    name: 'Token Registry',
    description: 'A registry of transactable tokens on the network',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0xf49089046f53f5d2e5f3513c1c32f5ff57d986e46309a42d2b249070e4e72c46',
    url: 'signaturereg',
    name: 'Method Registry',
    description: 'A registry of method signatures for lookups on transactions',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0'
  },
  {
    id: '0x058740ee9a5a3fb9f1cfa10752baec87e09cc45cd7027fd54708271aca300c75',
    url: 'githubhint',
    name: 'GitHub Hint',
    description: 'A mapping of GitHub URLs to hashes for use in contracts as references',
    author: 'Parity Team <admin@ethcore.io>',
    version: '1.0.0',
    secure: true
  }
];

export default class DappsStore {
  @observable apps = [];
  @observable hidden = [];
  @observable modalOpen = false;

  constructor (api) {
    this._api = api;

    this._readHiddenApps();
    this._fetch();
  }

  @computed get visible () {
    return this.apps.filter((app) => !this.hidden.includes(app.id));
  }

  @action openModal = () => {
    this.modalOpen = true;
  }

  @action closeModal = () => {
    this.modalOpen = false;
  }

  @action hideApp = (id) => {
    this.hidden = this.hidden.concat(id);
    this._writeHiddenApps();
  }

  @action showApp = (id) => {
    this.hidden = this.hidden.filter((_id) => _id !== id);
    this._writeHiddenApps();
  }

  _getHost (api) {
    return process.env.NODE_ENV === 'production'
      ? this._api.dappsUrl
      : '';
  }

  _fetch () {
    Promise
      .all([
        this._fetchLocal(),
        this._fetchRegistry()
      ])
      .then(([localApps, registryApps]) => {
        this.apps = []
          .concat(localApps)
          .concat(registryApps)
          .filter((app) => app.id)
          .sort((a, b) => (a.name || '').localeCompare(b.name || ''));
      })
      .catch((error) => {
        console.warn('DappStore:fetch', error);
      });
  }

  _fetchRegistry () {
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
        const appIds = appsInfo.map(([appId, owner]) => {
          return this._api.util.bytesToHex(appId);
        });

        return Promise
          .all([
            Promise.all(appIds.map((appId) => dappReg.getImage(appId))),
            Promise.all(appIds.map((appId) => dappReg.getContent(appId))),
            Promise.all(appIds.map((appId) => dappReg.getManifest(appId)))
          ])
          .then(([imageIds, contentIds, manifestIds]) => {
            return appIds.map((appId, index) => {
              const app = builtinApps.find((ba) => ba.id === appId) || {
                id: appId,
                contentHash: this._api.util.bytesToHex(contentIds[index]).substr(2),
                manifestHash: this._api.util.bytesToHex(manifestIds[index]).substr(2),
                type: 'network'
              };

              app.image = hashToImageUrl(imageIds[index]);
              app.type = app.type || 'builtin';

              return app;
            });
          });
      })
      .then((apps) => {
        return Promise
          .all(apps.map((app) => {
            return app.manifestHash
              ? this._fetchManifest(app.manifestHash)
              : null;
          }))
          .then((manifests) => {
            return apps.map((app, index) => {
              const manifest = manifests[index];

              if (manifest) {
                app.manifestHash = null;
                Object.keys(manifest)
                  .filter((key) => key !== 'id')
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
      .catch((error) => {
        console.warn('DappsStore:fetchRegistry', error);
      });
  }

  _fetchManifest (manifestHash, count = 0) {
    return fetch(`${this._getHost()}/api/content/${manifestHash}/`)
      .then((response) => {
        if (response.ok) {
          return response.json();
        }

        if (count < 1) {
          return this._fetchManifest(manifestHash, count + 1);
        }

        return null;
      })
      .catch(() => {
        if (count < 1) {
          return this._fetchManifest(manifestHash, count + 1);
        }

        return null;
      });
  }

  _fetchLocal () {
    return fetch(`${this._getHost()}/api/apps`)
      .then((response) => {
        return response.ok
          ? response.json()
          : [];
      })
      .then((localApps) => {
        return localApps
          .filter((app) => app && app.id && !['ui'].includes(app.id))
          .map((app) => {
            app.type = 'local';
            return app;
          });
      })
      .catch((error) => {
        console.warn('DappsStore:fetchLocal', error);
      });
  }

  _readHiddenApps () {
    const stored = localStorage.getItem('hiddenApps');

    if (stored) {
      try {
        this.hidden = JSON.parse(stored);
      } catch (error) {
        console.warn('DappsStore:readHiddenApps', error);
      }
    }
  }

  _writeHiddenApps () {
    localStorage.setItem('hiddenApps', JSON.stringify(this.hidden));
  }
}
