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
import { flatten } from 'lodash';

import * as abis from '~/contracts/abi';
import Contracts from '~/contracts';
import builtinJson from '~/views/Dapps/builtin.json';

import Dapp from './dappStore.js';
import { deleteDapp, registerDapp, updateDapp } from './utils';

import { api, trackRequest } from './parity';

const builtins = builtinJson.filter((app) => app.id);

let instance = null;

export default class DappsStore {
  @observable accounts = [];
  @observable apps = [];
  @observable contractOwner = null;
  @observable count = 0;
  @observable fee = new BigNumber(0);
  @observable isContractOwner = false;
  @observable isLoading = true;
  @observable transactions = {};

  _instanceGhh = null;
  _instanceReg = null;
  _registry = null;
  _startTime = Date.now();

  constructor () {
    this._loadDapps();
  }

  static get () {
    if (!instance) {
      instance = new DappsStore();
    }

    return instance;
  }

  createDappId () {
    return api.util.sha3(`${this._startTime}_${Date.now()}`);
  }

  @computed get ownedCount () {
    return this.ownDapps.length;
  }

  @computed get ownDapps () {
    return this.apps.filter((app) => app.isOwner);
  }

  @computed get otherDapps () {
    return this.apps.filter((app) => !app.isOwner);
  }

  @action sortApps = () => {
    // Sort dapps per name, then per id
    const sort = (a, b) => {
      if (a.name && b.name) {
        return a.name.localeCompare(b.name);
      }

      if (a.name) {
        return -1;
      }

      if (b.name) {
        return 1;
      }

      return a.id - b.id;
    };

    transaction(() => {
      const ownDapps = this.ownDapps.sort(sort);
      const otherDapps = this.otherDapps.sort(sort);

      this.apps = ownDapps.concat(otherDapps);
    });
  }

  @action setApps = (dapps) => {
    const filteredDapps = dapps.filter((dapp) => {
      return new BigNumber(dapp.id).gt(0);
    });

    transaction(() => {
      this.apps = filteredDapps;
      this.sortApps();
    });
  }

  @action refreshApp = (dappId) => {
    const dapp = this.apps.find((dapp) => dapp.id === dappId);

    this._loadDapp(dapp);
  }

  @action removeApp = (dappId) => {
    const dapps = this.apps.filter((dapp) => dapp.id !== dappId);

    this.setApps(dapps);
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

  @action setCount = (count) => {
    this.count = count;
    return count;
  }

  @action setFee = (fee) => {
    this.fee = fee;
    return fee;
  }

  @action setLoading = (loading) => {
    this.isLoading = loading;
    return loading;
  }

  @action updateTransaction = (requestId, nextData) => {
    const prevTransaction = this.transactions[requestId] || { requestId };
    const nextTransaction = {
      ...prevTransaction,
      hide: false,
      ...nextData
    };

    this.transactions = {
      ...this.transactions,
      [ requestId ]: nextTransaction
    };
  }

  fetchRegistryData (dapp) {
    const ownerAddress = (dapp.wip && dapp.wip.owner.address) || dapp.owner.address;

    this._registry.reverse
      .call({}, [ ownerAddress ])
      .then((name) => {
        if (!name) {
          return;
        }

        const key = api.util.sha3.text(name);

        return Promise
          .all([
            this._registry.get.call({}, [ key, 'IMG' ])
              .then((bytes) => api.util.bytesToHex(bytes))
              .then((hash) => this._instanceGhh.entries.call({}, [ hash ])),
            this._registry.get.call({}, [ key, 'CONTENT' ])
              .then((bytes) => api.util.bytesToHex(bytes))
              .then((hash) => this._instanceGhh.entries.call({}, [ hash ]))
          ])
          .then(([ imageGHH, contentGHH ]) => {
            const imageUrl = imageGHH[0];
            const contentUrl = contentGHH[0];

            return dapp.update({
              image: imageUrl,
              content: contentUrl
            });
          });
      });
  }

  register (dappId) {
    const dappRegInstance = this._instanceReg;
    const dappRegFee = this.fee;

    return registerDapp(dappId, dappRegInstance, dappRegFee)
      .then((request) => this.trackRequest(request, `Registering ${dappId}`))
      .then(() => this._loadDapps());
  }

  delete (dapp) {
    const dappRegInstance = this._instanceReg;

    return deleteDapp(dapp, dappRegInstance)
      .then((request) => this.trackRequest(request, `Deleting ${dapp.id}`))
      .then(() => this._loadDapps());
  }

  update (dappId, dappOwner, updates) {
    const dappRegInstance = this._instanceReg;
    const ghhRegInstance = this._instanceGhh;

    const promises = updateDapp(dappId, dappOwner, updates, dappRegInstance, ghhRegInstance);
    const handledPromises = promises.map((promise) => {
      return promise
        .then((requests) => {
          const requestPromises = flatten([].concat(requests))
            .filter((request) => request)
            .map((request) => this.trackRequest(request.id, request.name));

          return Promise.all(requestPromises);
        })
        .catch((error) => {
          const randomRequestId = api.util.sha3(Date.now()).slice(0, 5);

          this.updateTransaction(randomRequestId, { start: Date.now(), error });
        });
    });

    return Promise.all(handledPromises)
      .then(() => this._loadDapps());
  }

  trackRequest (requestId, name) {
    const statusCallback = (error, data) => {
      if (error) {
        return this.updateTransaction(requestId, { error });
      }

      return this.updateTransaction(requestId, data);
    };

    this.updateTransaction(requestId, { name, start: Date.now() });
    return trackRequest(requestId, statusCallback);
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
      .then((dappsInfo) => {
        const dapps = dappsInfo.reduce((dapps, [dappId, ownerAddress]) => {
          const id = api.util.bytesToHex(dappId);
          const owner = this.accounts.find((account) => account.address === ownerAddress);
          const isOwner = !!owner;
          const dapp = {
            id,
            owner: owner || { address: ownerAddress },
            isOwner
          };

          dapps[id] = dapp;
          return dapps;
        }, {});

        const promises = Object.values(dapps)
          // Only show dapps with id and owners
          .filter((dapp) => dapp.id && dapp.owner && !/^(0x)?0*$/.test(dapp.owner.address))
          .map((dapp) => this._loadDapp(dapp));

        return Promise.all(promises);
      })
      .then((dapps) => {
        this.setApps(dapps);
        this.sortApps();
        this.setLoading(false);
      })
      .catch((error) => {
        console.error('Store:loadDapps', error);
      });
  }

  _loadDapp = (dappData) => {
    const { id, owner, isOwner } = dappData;

    return Promise
      .all([
        this._loadMeta(id, 'CONTENT'),
        this._loadMeta(id, 'IMG'),
        this._loadMeta(id, 'MANIFEST')
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
              ._loadManifest(id, manifestHash, manifestUrl)
              .then((manifestContent) => {
                const content = {
                  hash: contentHash,
                  url: contentUrl
                };

                const image = {
                  hash: imageHash,
                  url: imageUrl
                };

                const manifest = {
                  content: manifestContent,
                  hash: manifestHash,
                  url: manifestUrl
                };

                return { content, image, manifest };
              });
          });
      })
      .catch((error) => {
        console.error('dappsStore::loadDapp', error);
        return {};
      })
      .then((data) => {
        const { content, image, manifest } = data;

        const dapp = new Dapp({
          contractOwner: this.contractOwner,
          isContractOwner: this.isContractOwner,
          id,
          content,
          image,
          manifest,
          owner,
          isOwner
        });

        return dapp;
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
    }

    if (!manifestHash) {
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
    return Contracts.create(api).registry
      .fetchContract()
      .then((contract) => {
        this._registry = contract.instance;
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
        console.log(`githubhint was found at ${ghhAddress}`);

        this._contractReg = api.newContract(abis.dappreg, dappregAddress);
        this._instanceReg = this._contractReg.instance;

        this._contractGhh = api.newContract(abis.githubhint, ghhAddress);
        this._instanceGhh = this._contractGhh.instance;
      })
      .catch((error) => {
        console.error('Store:attachContract', error);
      });
  }
}
