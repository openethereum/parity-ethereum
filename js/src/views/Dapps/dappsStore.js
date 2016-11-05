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
import { computed, observable } from 'mobx';

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
    version: '1.0.0'
  }
];

// TODO: This is just since we are moving gavcoin to its own repo, for a proper network solution
// we need to pull the network apps from the dapp registry. (Builtins & local apps unaffected)
// TODO: Manifest needs to be pulled from the content as well, however since the content may or may
// not be available locally (and refreshes work for index, and will give a 503), we are putting it
// in here. This part needs to be cleaned up.
const networkApps = [
  {
    id: '0xd798a48831b4ccdbc71de206a1d6a4aa73546c7b6f59c22a47452af414dc64d6',
    name: 'GAVcoin',
    description: 'Manage your GAVcoins, the hottest new property in crypto',
    author: 'Gavin Wood',
    version: '1.0.0'
  }
];

function getHost (api) {
  return process.env.NODE_ENV === 'production'
    ? api.dappsUrl
    : '';
}

export default class DappsStore {
  @observable apps = [];
  @observable hidden = [];

  constructor (api) {
    this._api = api;

    this.readHiddenApps();
    this.fetch();
  }

  @computed get visibleApps () {
    return this.apps.filter((app) => !this.hidden.includes(app.id));
  }

  fetch () {
    fetch(`${getHost(this._api)}/api/apps`)
      .then((response) => {
        return response.ok
          ? response.json()
          : [];
      })
      .catch((error) => {
        console.warn('DappsStore:fetch', error);
        return [];
      })
      .then((_localApps) => {
        const localApps = _localApps
          .filter((app) => !['ui'].includes(app.id))
          .map((app) => {
            app.local = true;
            return app;
          });

        return this._api.ethcore
          .registryAddress()
          .then((registryAddress) => {
            if (new BigNumber(registryAddress).eq(0)) {
              return [];
            }

            const _builtinApps = builtinApps
              .map((app) => {
                app.builtin = true;
                return app;
              });

            return networkApps
              .map((app) => {
                app.network = true;
                return app;
              })
              .concat(_builtinApps);
          })
          .then((registryApps) => {
            this.apps = registryApps
              .concat(localApps)
              .sort((a, b) => (a.name || '').localeCompare(b.name || ''));
            this.loadImages();
          });
      })
      .catch((error) => {
        console.warn('DappsStore:fetch', error);
      });
  }

  loadImages () {
    const { dappReg } = Contracts.get();

    return Promise
      .all(this.apps.map((app) => dappReg.getImage(app.id)))
      .then((images) => {
        this.apps = images
          .map(hashToImageUrl)
          .map((image, index) => Object.assign({}, this.apps[index], { image }));

        const _networkApps = this.apps.filter((app) => app.network);

        return Promise
          .all(_networkApps.map((app) => dappReg.getContent(app.id)))
          .then((content) => {
            const networkApps = content.map((_contentHash, index) => {
              const networkApp = _networkApps[index];
              const contentHash = this._api.util.bytesToHex(_contentHash).substr(2);
              const app = this.apps.find((_app) => _app.id === networkApp.id);

              console.log(`found content for ${app.id} at ${contentHash}`);
              return Object.assign({}, app, { contentHash });
            });

            this.apps = this.apps.map((app) => {
              return Object.assign({}, networkApps.find((napp) => app.id === napp.id) || app);
            });
          });
      })
      .catch((error) => {
        console.warn('DappsStore:loadImages', error);
      });
  }

  manifest (app, contentHash) {
    fetch(`${getHost(this._api)}/${contentHash}/manifest.json`)
      .then((response) => {
        return response.ok
          ? response.json()
          : {};
      })
      .then((manifest) => {
        Object.keys.forEach((key) => {
          app[key] = manifest[key];
        });

        return app;
      })
      .catch((error) => {
        console.warn('DappsStore:manifest', error);
      });
  }

  readHiddenApps () {
    const stored = localStorage.getItem('hiddenApps');

    if (stored) {
      try {
        this.hidden = JSON.parse(stored);
      } catch (error) {
        console.warn('DappsStore:readHiddenApps', error);
      }
    }
  }

  writeHiddenApps () {
    localStorage.setItem('hiddenApps', JSON.stringify(this.hidden));
  }

  hideApp (id) {
    this.hidden = this.hidden.concat(id);
    this.writeHiddenApps();
  }

  showApp (id) {
    this.hidden = this.hidden.filter((_id) => _id !== id);
    this.writeHiddenApps();
  }
}
