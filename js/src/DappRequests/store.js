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
import store from 'store';

import { sha3 } from '@parity/api/lib/util/sha3';

import methodGroups, { methodGroupFromMethod } from './methodGroups';

const LS_PERMISSIONS = '_parity::dapps::methods';

export default class Store {
  @observable permissions = {}; // Maps `${method}:${appId}` to true/false
  @observable requests = {}; // Maps `${method}:${appId}` to request (if multiple requests for same app+method, then only store last)
  @observable tokens = {}; // Maps token to appId

  middleware = [];

  constructor (provider) {
    this.provider = provider;
    this.permissions = store.get(LS_PERMISSIONS) || {};

    window.addEventListener('message', this.receiveMessage, false);
  }

  @computed get hasRequests () {
    return Object.keys(this.requests).length !== 0;
  }

  @computed get groupedRequests () {
    // Group by appId on top level, and by methodGroup on 2nd level
    return Object.values(this.requests).reduce((accumulator, request) => {
      const { data: { method, params, token } } = request;
      const appId = this.tokens[token];

      accumulator[appId] = accumulator[appId] || {};

      const methodGroup = methodGroupFromMethod[method || params[0]]; // Get the methodGroup the current request belongs to

      accumulator[appId][methodGroup] = accumulator[appId][methodGroup] || [];
      accumulator[appId][methodGroup].push(request);
      return accumulator;
    }, {});
  }

  @action createToken = appId => {
    const token = sha3(`${appId}:${Date.now()}`);

    this.tokens = Object.assign({}, this.tokens, {
      [token]: appId
    });

    return token;
  };

  @action queueRequest = request => {
    const { data: { method, token } } = request;
    const appId = this.tokens[token];

    // Add a new request in this.requests
    this.requests = {
      ...this.requests,
      [`${method}:${appId}`]: request
    };
  };

  @action approveRequest = (method, appId) => {
    const { source, data } = this.requests[`${method}:${appId}`];

    this.addAppPermission(method, appId);
    this.removeRequest(method, appId);

    if (data.api) {
      this.executePubsubCall(data, source);
    } else {
      this.executeMethodCall(data, source);
    }
  };

  @action approveRequestGroup = (groupId, appId) => {
    // Get methods of this requestGroup
    const { methods } = methodGroups[groupId];

    methods.forEach(method => {
      if (!this.requests[`${method}:${appId}`]) {
        return;
      }

      this.approveRequest(method, appId);
    });
  };

  @action rejectRequest = (method, appId) => {
    const { source, data } = this.requests[`${method}:${appId}`];

    this.removeRequest(method, appId);
    this.rejectMessage(source, data);
  };

  @action rejectRequestGroup = (groupId, appId) => {
    // Get methods of this requestGroup
    const { methods } = methodGroups[groupId];

    methods.forEach(method => {
      if (!this.requests[`${method}:${appId}`]) {
        return;
      }

      this.rejectRequest(method, appId);
    });
  };

  @action removeRequest = (method, appId) => {
    delete this.requests[`${method}:${appId}`];
    this.requests = { ...this.requests };
  };

  @action rejectMessage = (source, { id, from, method, token }) => {
    if (!source) {
      return;
    }

    source.postMessage(
      {
        error: `Method ${method} not allowed`,
        id,
        from: 'shell',
        result: null,
        to: from,
        token
      },
      '*'
    );
  };

  @action addAppPermission = (method, appId) => {
    this.permissions = Object.assign({}, this.permissions, {
      [`${method}:${appId}`]: true
    });
    this.savePermissions();
  };

  @action setPermissions = _permissions => {
    const permissions = {};

    Object.keys(_permissions).forEach(id => {
      permissions[id] = !!_permissions[id];
    });

    this.permissions = Object.assign({}, this.permissions, permissions);
    this.savePermissions();

    return true;
  };

  addMiddleware (middleware) {
    if (!middleware || typeof middleware !== 'function') {
      throw new Error('Interceptor middleware does not implement a function');
    }

    this.middleware.push(middleware);

    return true;
  }

  hasValidToken = (method, appId, token) => {
    if (!token) {
      return method === 'shell_requestNewToken';
    }

    return this.tokens[token] === appId;
  };

  hasTokenPermission = (method, token) => {
    return this.hasAppPermission(method, this.tokens[token]);
  };

  hasAppPermission = (method, appId) => {
    return this.permissions[`${method}:${appId}`] || false;
  };

  savePermissions = () => {
    store.set(LS_PERMISSIONS, this.permissions);
  };

  _methodCallbackPost = (id, from, source, token) => {
    return (error, result) => {
      if (!source) {
        return;
      }

      source.postMessage(
        {
          error: error ? error.message : null,
          id,
          from: 'shell',
          to: from,
          result,
          token
        },
        '*'
      );
    };
  };

  executePubsubCall = ({ api, id, from, token, params }, source) => {
    const callback = this._methodCallbackPost(id, from, source, token);

    this.provider.subscribe(api, callback, params).then((result, error) => {
      this._methodCallbackPost(id, from, source, token)(null, result);
    });
  };

  executeMethodCall = ({ id, from, method, params, token }, source) => {
    const callback = this._methodCallbackPost(id, from, source, token);
    const isHandled = this.middleware.find(middleware =>
      middleware(from, method, params, callback)
    );

    if (!isHandled) {
      this.provider.send(method, params, callback);
    }
  };

  receiveMessage = ({ data, origin, source }) => {
    if (!data) {
      return;
    }

    const { from, method, to, token, params, api, subId, id } = data;

    if (to !== 'shell' || !from || from === 'shell') {
      return;
    }

    if (!this.hasValidToken(method, from, token)) {
      this.rejectMessage(source, data);
      return;
    }

    if (
      (method &&
        methodGroupFromMethod[method] &&
        !this.hasTokenPermission(method, token)) ||
      (api &&
        methodGroupFromMethod[params[0]] &&
        !this.hasTokenPermission(method, token))
    ) {
      this.queueRequest({ data, origin, source });
      return;
    }

    if (api) {
      this.executePubsubCall(data, source);
    } else if (subId) {
      subId === '*'
        ? this.provider
          .unsubscribeAll()
          .then(v =>
            this._methodCallbackPost(id, from, source, token)(null, v)
          )
        : this.provider
          .unsubscribe(subId)
          .then(v =>
            this._methodCallbackPost(id, from, source, token)(null, v)
          );
    } else {
      this.executeMethodCall(data, source);
    }
  };

  static instance = null;

  static create (provider) {
    if (!Store.instance) {
      Store.instance = new Store(provider);
    }

    return Store.instance;
  }

  static get () {
    return Store.instance;
  }
}
