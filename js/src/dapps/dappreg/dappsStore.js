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

import BigNumber from 'bignumber.js';
import { action, computed, observable, transaction } from 'mobx';

import * as abis from '~/contracts/abi';
import builtins from '~/views/Dapps/builtin.json';

import { api } from './parity';

let instance = null;

export default class DappsStore {
  @observable accounts = [];
  @observable apps = [];
  @observable contractOwner = null;
  @observable currentAccount = null;
  @observable currentApp = null;
  @observable count = 0;
  @observable fee = new BigNumber(0);
  @observable isContractOwner = false;
  @observable isEditing = false;
  @observable isLoading = true;
  @observable isNew = false;
  @observable wipApp = null;

  _startTime = Date.now();

  constructor () {
    this._loadDapps();
  }

  static instance () {
    if (!instance) {
      instance = new DappsStore();
    }

    return instance;
  }

  @computed get canSave () {
    const app = this.wipApp;

    const hasError = app.contentError || app.imageError || app.manifestError;
    const isDirty = this.isNew || app.contentChanged || app.imageChanged || app.manifestChanged;
    const isEditMode = this.isEditing || this.isNew;

    return isEditMode && isDirty && !hasError;
  }

  @computed get isCurrentEditable () {
    return !!this.accounts.find((account) => account.address === this.currentApp.owner);
  }

  @computed get ownedCount () {
    return (this.apps.filter((app) => app.isOwner) || []).length;
  }

  @action copyToWip = () => {
    let wipApp;

    if (this.isNew) {
      wipApp = {
        id: api.util.sha3(`${this._startTime}_${Date.now()}`),
        contentHash: null,
        contentUrl: null,
        imageHash: null,
        imageUrl: null,
        manifestHash: null,
        manifestUrl: null
      };
    } else {
      const app = this.currentApp;

      wipApp = {
        id: app.id,
        contentHash: app.contentHash,
        contentUrl: app.contentUrl,
        imageHash: app.imageHash,
        imageUrl: app.imageUrl,
        manifestHash: app.manifestHash,
        manifestUrl: app.manifestUrl,
        owner: app.owner,
        ownerName: app.ownerName
      };
    }

    this.wipApp = Object.assign(wipApp, {
      contentChanged: false,
      contentError: null,
      imageChanged: false,
      imageError: null,
      manifestChanged: false,
      manifestError: null
    });

    return this.wipApp;
  }

  @action editWip = (details) => {
    if (this.isNew || this.isEditing) {
      transaction(() => {
        Object
          .keys(details)
          .forEach((key) => {
            this.wipApp[key] = details[key];
          });
      });
    }

    return this.wipApp;
  }

  @action sortApps = (apps = this.apps) => {
    transaction(() => {
      const ownApps = apps
        .filter((app) => app.isOwner)
        .sort((a, b) => a.name.localeCompare(b.name));
      const otherApps = apps
        .filter((app) => !app.isOwner)
        .sort((a, b) => a.name.localeCompare(b.name));

      this.apps = ownApps.concat(otherApps);

      if (this.apps.length) {
        this.currentApp = this.apps[0];
      }
    });
  }

  @action setApps = (apps) => {
    this.sortApps(apps.filter((app) => {
      const bnid = new BigNumber(app.id);

      return bnid.gt(0);
    }));

    return this.apps;
  }

  @action _addApp = (app) => {
    transaction(() => {
      this.setApps(this.apps.concat([app]));
      this.setCurrentApp(app.id);
    });
  }

  @action addApp = (appId, account) => {
    this
      ._loadDapp({
        id: appId,
        isOwner: true,
        name: `- ${appId}`,
        owner: account.address,
        ownerName: account.name
      })
      .then(this._addApp);
  }

  @action refreshApp = (appId) => {
    this._loadDapp(this.apps.find((app) => app.id === appId));
  }

  @action removeApp = (appId) => {
    this.setApps(this.apps.filter((app) => app.id !== appId));
  }

  @action setAppInfo = (app, info) => {
    transaction(() => {
      Object.keys(info).forEach((key) => {
        app[key] = info[key];
      });
    });

    return app;
  }

  @action setAccounts = (accountsInfo) => {
    transaction(() => {
      this.accounts = Object
        .keys(accountsInfo)
        .map((address) => {
          const account = accountsInfo[address];

          account.address = address;
          return account;
        });

      this.currentAccount = this.accounts[0];
    });

    return this.accounts;
  }

  @action setContractOwner = (contractOwner) => {
    transaction(() => {
      this.contractOwner = contractOwner;
      this.isContractOwner = !!this.accounts.find((account) => account.address === contractOwner);
    });
    return contractOwner;
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
    transaction(() => {
      this.isEditing = mode;
      this.copyToWip();
    });

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
    transaction(() => {
      this.isNew = mode;
      this.copyToWip();
    });

    return mode;
  }

  lookupHash (hash) {
    return this._retrieveUrl(hash);
  }

  _getCount () {
    return this._instanceReg
      .count.call()
      .then((count) => {
        this.setCount(count.toNumber());
      })
      .catch((error) => {
        console.error('Store:getCount', error);
      });
  }

  _getFee () {
    return this._instanceReg
      .fee.call()
      .then(this.setFee)
      .catch((error) => {
        console.error('Store:getFee', error);
      });
  }

  _getOwner () {
    return this._instanceReg
      .owner.call()
      .then(this.setContractOwner)
      .catch((error) => {
        console.error('Store:getOwner', error);
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
        this._getFee(),
        this._getOwner()
      ]))
      .then(() => {
        const promises = [];

        for (let index = 0; index < this.count; index++) {
          promises.push(this._instanceReg.at.call({}, [index]));
        }

        return Promise.all(promises);
      })
      .then((appsInfo) => {
        return Promise.all(
          this
            .setApps(appsInfo.map(([appId, owner]) => {
              const isOwner = !!this.accounts.find((account) => account.address === owner);
              const account = this.accounts.find((account) => account.address === owner);
              const id = api.util.bytesToHex(appId);

              return {
                id,
                owner,
                ownerName: account ? account.name : owner,
                isOwner,
                name: `- ${id}`
              };
            }))
            .map(this._loadDapp)
        );
      })
      .then(() => {
        this.sortApps();
        this.setLoading(false);
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
                  name: (manifest && manifest.name) || `- ${app.id}`
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
    return this._instanceReg
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
      .accountsInfo()
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
        this._contractReg = api.newContract(abis.dappreg, dappregAddress);
        this._instanceReg = this._contractReg.instance;
        console.log(`githubhint was found at ${ghhAddress}`);
        this._contractGhh = api.newContract(abis.githubhint, ghhAddress);
        this._instanceGhh = this._contractGhh.instance;
      })
      .catch((error) => {
        console.error('Store:attachContract', error);
      });
  }
}
