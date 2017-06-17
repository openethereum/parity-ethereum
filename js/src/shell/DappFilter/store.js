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

import { action, computed, observable } from 'mobx';

let nextQueueId = 0;

export default class Store {
  @observable permissions = [];
  @observable requests = [];

  constructor (provider, permissions) {
    this.permissions = permissions;
    this.provider = provider;

    window.addEventListener('message', this.receiveMessage, false);
  }

  @computed get hasRequests () {
    return this.requests.length !== 0;
  }

  @action removeRequest = (_queueId) => {
    this.requests = this.requests.filter(({ queueId }) => queueId !== _queueId);
  }

  @action queueRequest = (request) => {
    let queueId = ++nextQueueId;

    this.requests = this.requests.concat([{ queueId, request }]);
  }

  @action approveSingleRequest = ({ queueId, request: { data, source } }) => {
    this.removeRequest(queueId);
    this.executeOnProvider(data, source);
  }

  @action approveRequest = (queueId, approveAll) => {
    const queued = this.findRequest(queueId);

    if (approveAll) {
      const { request: { data: { method, token } } } = queued;
      const requests = this.findMatchingRequests(method, token);

      this.addTokenPermission(method, token);
      requests.forEach(this.approveSingleRequest);
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

  @action addTokenPermission = (method, token) => {
    this.permissions.tokens[token] = Object.assign({ [method]: true }, this.permissions.tokens[token] || {});
  }

  @action setPermissions = (permissions) => {
    this.permissions = permissions;
  }

  findRequest (_queueId) {
    return this.requests.find(({ queueId }) => queueId === _queueId);
  }

  findMatchingRequests (_method, _token) {
    return this.requests.filter(({ request: { data: { method, token } } }) => method === _method && token === _token);
  }

  executeOnProvider = ({ id, from, method, params, token }, source) => {
    this.provider.send(method, params, (error, result) => {
      source.postMessage({
        error: error
          ? error.message
          : null,
        id,
        from: 'shell',
        result,
        token
      }, '*');
    });
  }

  receiveMessage = ({ data, origin, source }) => {
    const { from, method, token } = data;

    if (from === 'shell' || from !== token) {
      return;
    }

    if (this.permissions.filtered.includes(method)) {
      if (!this.permissions.tokens[token] || !this.permissions.tokens[token][method]) {
        this.queueRequest({ data, origin, source });
        return;
      }
    }

    this.executeOnProvider(data, source);
  }

  static instance = null;

  static create (provider, permissions) {
    Store.instance = new Store(provider, permissions);
  }

  static get () {
    return Store.instance;
  }
}
