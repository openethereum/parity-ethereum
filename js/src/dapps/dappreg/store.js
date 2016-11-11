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
import { action, computed, observable } from 'mobx';

import * as abis from '../../contracts/abi';
import builtins from '../../views/Dapps/builtin.json';

import { api } from './parity';

let instance = null;

export default class Store {
  @observable accounts = [];
  @observable addresses = [];
  @observable apps = [];
  @observable currentAccount = null;
  @observable currentApp = null;
  @observable count = 0;
  @observable fee = new BigNumber(0);
  @observable isEditing = false;
  @observable isLoading = true;
  @observable isNew = false;

  constructor () {
    this._startTime = Date.now();

    this._loadDapps();
  }

  static instance () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }

  @computed get getNewId () {
    return api.util.sha3(`${this._startTime}_${Date.now()}`);
  }

  @computed get isCurrentEditable () {
    return !!this.accounts.find((account) => account.address === this.currentApp.owner);
  }

  @computed get ownedCount () {
    return (this.apps.filter((app) => app.isOwner) || []).length;
  }

  @action setApps = (apps) => {
    this.apps = apps
      .sort((a, b) => a.name.localeCompare(b.name))
      .sort((a, b) => {
        return a.isOwner === b.isOwner
          ? 0
          : (a.isOwner ? -1 : 1);
      });

    this.currentApp = this.apps[0];
    return this.apps;
  }

  @action setAppInfo = (app, info) => {
    Object.keys(info).forEach((key) => {
      app[key] = info[key];
    });
    return app;
  }

  @action setAccounts = (accountsInfo) => {
    this.addresses = Object
      .keys(accountsInfo)
      .map((address) => {
        const account = accountsInfo[address];
        account.address = address;
        return account;
      });

    this.accounts = this.addresses.filter((account) => account.uuid);
    this.currentAccount = this.accounts[0];
    return this.accounts;
  }

  @action setCurrentApp = (id) => {
    this.currentApp = this.apps.find((app) => app.id === id);
    return this.currentApp;
  }

  @action setCurrentAccount = (address) => {
    this.currentAccount = this.accounts.find((account) => account.address === address);
    return this.currentAccount;
  }

  @action setCount = (count) => {
    this.count = count;
    return count;
  }

  @action setEditing = (mode) => {
    this.isEditing = mode;
    return mode;
  }

  @action setFee = (fee) => {
    this.fee = fee;
    return fee;
  }

  @action setLoading = (loading) => {
    this.isLoading = loading;
    return loading;
  }

  @action setNew = (mode) => {
    this.isNew = mode;
    return mode;
  }

  _getCount () {
    return this._instanceDr
      .count.call()
      .then((count) => {
        this.setCount(count.toNumber());
      })
      .catch((error) => {
        console.error('Store:getCount', error);
      });
  }

  _getFee () {
    return this._instanceDr
      .fee.call()
      .then(this.setFee)
      .catch((error) => {
        console.error('Store:getFee', error);
      });
  }

  _loadDapps () {
    return this._loadRegistry()
      .then(() => Promise.all([
        this._attachContracts(),
        this._loadAccounts()
      ]))
      .then(() => Promise.all([
        this._getCount(),
        this._getFee()
      ]))
      .then(() => {
        const promises = [];

        for (let index = 0; index < this.count; index++) {
          promises.push(this._instanceDr.at.call({}, [index]));
        }

        return Promise.all(promises);
      })
      .then((appsInfo) => {
        return Promise.all(
          this
            .setApps(appsInfo.map(([appId, owner]) => {
              const isOwner = !!this.accounts.find((account) => account.address === owner);
              const account = this.addresses.find((account) => account.address === owner);

              return {
                owner,
                ownerName: account ? account.name : owner,
                isOwner,
                name: '-',
                id: api.util.bytesToHex(appId)
              };
            }))
            .map(this._loadDapp)
        );
      })
      .then(() => {
        this.setLoading(this.count === 0);
      })
      .catch((error) => {
        console.error('Store:loadDapps', error);
      });
  }

  _loadDapp = (app) => {
    return Promise
      .all([
        this._loadMeta(app.id, 'CONTENT'),
        this._loadMeta(app.id, 'IMG'),
        this._loadMeta(app.id, 'MANIFEST')
      ])
      .then(([contentHash, imageHash, manifestHash]) => {
        return Promise
          .all([
            this._retrieveUrl(contentHash),
            this._retrieveUrl(imageHash),
            this._retrieveUrl(manifestHash)
          ])
          .then(([contentUrl, imageUrl, manifestUrl]) => {
            return this
              ._loadManifest(app.id, manifestHash)
              .then((manifest) => {
                this.setAppInfo(app, {
                  manifest,
                  manifestHash,
                  manifestUrl,
                  contentHash,
                  contentUrl,
                  imageHash,
                  imageUrl,
                  name: manifest ? manifest.name : '-'
                });

                return app;
              });
          });
      })
      .catch((error) => {
        console.error('Store:loadDapp', error);
      });
  }

  _loadMeta (appId, key) {
    return this._instanceDr
      .meta.call({}, [appId, key])
      .then((meta) => {
        const hash = api.util.bytesToHex(meta);
        const bnhash = new BigNumber(hash);

        return bnhash.gt(0)
          ? hash
          : null;
      })
      .catch((error) => {
        console.error('Store:loadMeta', error);
        return null;
      });
  }

  _loadManifest (appId, manifestHash) {
    const builtin = builtins.find((app) => app.id === appId);

    if (builtin) {
      return Promise.resolve(builtin);
    } else if (!manifestHash) {
      return Promise.resolve(null);
    }

    return fetch(`/api/content/${manifestHash.substr(2)}/`, { redirect: 'follow', mode: 'cors' })
      .then((response) => {
        return response.ok
          ? response.json()
          : null;
      })
      .catch((error) => {
        console.error('Store:loadManifest', error);
        return null;
      });
  }

  _retrieveUrl (urlHash) {
    if (!urlHash) {
      return Promise.resolve(null);
    }

    return this._instanceGhh
      .entries.call({}, [urlHash])
      .then(([repo, _commit, owner]) => {
        const bnowner = new BigNumber(owner);

        if (bnowner.eq(0)) {
          return null;
        }

        const commit = api.util.bytesToHex(_commit);
        const bncommit = new BigNumber(commit);

        if (bncommit.eq(0)) {
          return repo;
        } else {
          return `https://codeload.github.com/${repo}/zip/${commit.substr(2)}`;
        }
      })
      .catch((error) => {
        console.error('Store:retriveUrl', error);
        return null;
      });
  }

  _loadAccounts () {
    return api.parity
      .accounts()
      .then(this.setAccounts)
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

  _attachContracts () {
    return Promise
      .all([
        this._registry.getAddress.call({}, [api.util.sha3('dappreg'), 'A']),
        this._registry.getAddress.call({}, [api.util.sha3('githubhint'), 'A'])
      ])
      .then(([dappregAddress, ghhAddress]) => {
        console.log(`dappreg was found at ${dappregAddress}`);
        this._contractDr = api.newContract(abis.dappreg, dappregAddress);
        this._instanceDr = this._contractDr.instance;
        console.log(`githubhint was found at ${ghhAddress}`);
        this._contractGhh = api.newContract(abis.githubhint, ghhAddress);
        this._instanceGhh = this._contractGhh.instance;
      })
      .catch((error) => {
        console.error('Store:attachContract', error);
      });
  }
}
