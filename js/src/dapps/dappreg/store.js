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
import * as abis from '../../contracts/abi';

import { api } from './parity';

let instance = null;

export default class Store {
  @observable loading = true;
  @observable count = 0;
  @observable apps = [];

  constructor () {
    this._startupTime = Date.now();

    this._loadAccounts();
    this._loadDapps();
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }

  @computed getNewId () {
    return api.util.sha3(`${this._startupTime}_${Date.now()}_${Math.random()}`);
  }

  @action _getCount () {
    return this._instance
      .count()
      .then((count) => {
        this.count = count.toNumber();
      })
      .catch((error) => {
        console.error('Store:getCount', error);
      });
  }

  @action _getFee () {
    return this._instance
      .fee()
      .then((fee) => {
        this.fee = fee;
      })
      .catch((error) => {
        console.error('Store:getFee', error);
      });
  }

  @action _loadDapps () {
    return this._loadRegistry()
      .then(() => this._attachContract())
      .then(() => Promise.all([
        this._getCount(),
        this._getFee()
      ]))
      .then(() => {
        const promises = [];

        for (let index = 0; index < this.count; index++) {
          promises.push(this._instance.at(index));
        }

        return Promise.all(promises);
      })
      .then((appsInfo) => {
        this.apps = appsInfo.map(([appId, owner]) => {
          return { owner, id: api.util.bytesToHex(appId) };
        });

        return Promise.all(this.apps.map(this._loadDapp));
      })
      .then(() => {
        this.loading = this.count === 0;
      })
      .catch((error) => {
        console.error('Store:loadDapps', error);
      });
  }

  @action _loadDapp (app) {
    return Promise
      .all([
        this._loadMeta(app.id, 'CONTENT'),
        this._loadMeta(app.id, 'IMG'),
        this._loadMeta(app.id, 'MANIFEST')
      ])
      .then(([contentHash, imageHash, manifestHash]) => {
        return this
          ._loadManifest(manifestHash)
          .then((manifest) => {
            app.manifest = manifest;
            app.contentHash = contentHash;
            app.imageHash = imageHash;
            app.manifestHash = manifestHash;

            return app;
          });
      })
      .catch((error) => {
        console.error('Store:loadDapp', error);
      });
  }

  _loadMeta (appId, key) {
    return this._instance
      .meta.call({}, [appId, key])
      .then((meta) => api.util.bytesToHex(meta).substr(2))
      .catch((error) => {
        console.error('Store:loadMeta', error);
        return null;
      });
  }

  _loadManifest (manifestHash) {
    return fetch(`/api/content/${manifestHash}/`, { redirect: 'follow', mode: 'cors' })
      .then((response) => {
        return response.ok
          ? response.json()
          : null;
      })
      .catch((error) => {
        console.warn('Store:loadManifest', error);
        return null;
      });
  }

  @action _loadAccounts () {
    return api.parity
      .accounts()
      .then((accountsInfo) => {
        return Object
          .keys(accountsInfo)
          .filter((address) => accountsInfo[address].uuid)
          .map((address) => {
            const account = accountsInfo[address];
            account.address = address;
            return account;
          });
      })
      .then((accounts) => {
        this.accounts = accounts;
      })
      .catch((error) => {
        console.error('Store:loadAccounts', error);
      });
  }

  _loadRegistry () {
    return api.parity
      .registryAddress()
      .then((registryAddress) => {
        console.log(`the registry was found at ${registryAddress}`);
        this._registry = api.newContract(abis.registry, registryAddress).instance;
      })
      .catch((error) => {
        console.error('Store:loadRegistry', error);
      });
  }

  _attachContract () {
    return this._registry
      .getAddress.call({}, [api.util.sha3('dappreg'), 'A'])
      .then((dappregAddress) => {
        console.log(`dappreg was found at ${dappregAddress}`);
        this._contract = api.newContract(abis.dappreg, dappregAddress);
        this._instance = this._contract.instance;
        console.log(this._instance);
        
      })
      .catch((error) => {
        console.error('Store:attachContract', error);
      });
  }
}
