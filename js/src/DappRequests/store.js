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

import { flatten } from 'lodash';
import { action, computed, observable } from 'mobx';
import store from 'store';

import { sha3 } from '@parity/api/util/sha3';

import VisibleStore from '../Dapps/dappsStore';
import filteredRequests from './filteredRequests';

const LS_PERMISSIONS = '_parity::dapps::methods';

let nextQueueId = 0;

export default class Store {
  @observable permissions = {};
  @observable requests = [];
  @observable tokens = {};

  constructor (provider) {
    this.provider = provider;
    this.permissions = store.get(LS_PERMISSIONS) || {};

    window.addEventListener('message', this.receiveMessage, false);
  }

  @computed get hasRequests () {
    return this.requests.length !== 0;
  }

  @computed get squashedRequests () {
    const duplicates = {};

    return this.requests.filter(({ request: { data: { method, token } } }) => {
      const section = this.getFilteredSectionName(method);
      const id = `${token}:${section}`;

      if (!duplicates[id]) {
        duplicates[id] = true;
        return true;
      }

      return false;
    });
  }

  @action createToken = (appId) => {
    const token = sha3(`${appId}:${Date.now()}`);

    this.tokens = Object.assign({}, this.tokens, {
      [token]: appId
    });

    return token;
  }

  @action removeRequest = (_queueId) => {
    this.requests = this.requests.filter(({ queueId }) => queueId !== _queueId);
  }

  @action queueRequest = (request) => {
    const appId = this.tokens[request.data.from];
    let queueId = ++nextQueueId;

    this.requests = this.requests.concat([{ appId, queueId, request }]);
  }

  @action addTokenPermission = (method, token) => {
    const id = `${method}:${this.tokens[token]}`;

    this.permissions = Object.assign({}, this.permissions, {
      [id]: true
    });
    this.savePermissions();
  }

  @action approveSingleRequest = ({ queueId, request: { data, source } }) => {
    this.removeRequest(queueId);
    this.executeMethodCall(data, source);
  }

  @action approveRequest = (queueId, approveAll) => {
    const queued = this.findRequest(queueId);

    if (approveAll) {
      const { request: { data: { method, token } } } = queued;

      this.getFilteredSection(method).methods.forEach((m) => {
        this.addTokenPermission(m, token);
        this.findMatchingRequests(m, token).forEach(this.approveSingleRequest);
      });
    } else {
      this.approveSingleRequest(queued);
    }
  }

  @action rejectRequest = (queueId) => {
    const { request: { data: { id, method, token }, source } } = this.findRequest(queueId);

    this.removeRequest(queueId);
    source.postMessage({
      error: `Method ${method} not allowed`,
      id,
      from: 'shell',
      result: null,
      token
    }, '*');
  }

  @action setPermissions = (_permissions) => {
    const permissions = {};

    Object.keys(_permissions).forEach((id) => {
      permissions[id] = !!_permissions[id];
    });

    this.permissions = Object.assign({}, this.permissions, permissions);
    this.savePermissions();

    return true;
  }

  hasTokenPermission = (method, token) => {
    return this.hasAppPermission(method, this.tokens[token]);
  }

  hasAppPermission = (method, appId) => {
    return this.permissions[`${method}:${appId}`] || false;
  }

  savePermissions = () => {
    store.set(LS_PERMISSIONS, this.permissions);
  }

  findRequest (_queueId) {
    return this.requests.find(({ queueId }) => queueId === _queueId);
  }

  findMatchingRequests (_method, _token) {
    return this.requests.filter(({ request: { data: { method, token } } }) => method === _method && token === _token);
  }

  _methodCallbackPost = (id, source, token) => {
    return (error, result) => {
      source.postMessage({
        error: error
          ? error.message
          : null,
        id,
        from: 'shell',
        result,
        token
      }, '*');
    };
  }

  executeMethodCall = ({ id, from, method, params, token }, source) => {
    const visibleStore = VisibleStore.get();
    const callback = this._methodCallbackPost(id, source, token);

    switch (method) {
      case 'shell_getApps':
        const [displayAll] = params;

        return callback(null, displayAll
          ? visibleStore.allApps.slice()
          : visibleStore.visibleApps.slice()
        );

      case 'shell_getFilteredMethods':
        return callback(null, flatten(
          Object
            .keys(filteredRequests)
            .map((key) => filteredRequests[key].methods)
        ));

      case 'shell_getMethodPermissions':
        return callback(null, this.permissions);

      case 'shell_setAppVisibility':
        const [appId, visibility] = params;

        return callback(null, visibility
          ? visibleStore.showApp(appId)
          : visibleStore.hideApp(appId)
        );

      case 'shell_setMethodPermissions':
        const [permissions] = params;

        return callback(null, this.setPermissions(permissions));

      default:
        return this.provider.send(method, params, callback);
    }
  }

  getFilteredSectionName = (method) => {
    return Object.keys(filteredRequests).find((key) => {
      return filteredRequests[key].methods.includes(method);
    });
  }

  getFilteredSection = (method) => {
    return filteredRequests[this.getFilteredSectionName(method)];
  }

  receiveMessage = ({ data, origin, source }) => {
    if (!data) {
      return;
    }

    const { from, method, token } = data;

    if (!from || from === 'shell' || from !== token) {
      return;
    }

    if (this.getFilteredSection(method) && !this.hasTokenPermission(method, token)) {
      this.queueRequest({ data, origin, source });
      return;
    }

    this.executeMethodCall(data, source);
  }

  static instance = null;

  static create (provider) {
    if (!Store.instance) {
      Store.instance = new Store(provider, {});
    }

    return Store.instance;
  }

  static get () {
    return Store.instance;
  }
}
