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

import EventEmitter from 'eventemitter3';

import Contract from './contract';
import { PromiseProvider, Http as HttpProvider, PostMessage as PostMessageProvider, WsSecure as WsSecureProvider } from './provider';
import { Http as HttpTransport, WsSecure as WsSecureTransport } from './transport';

import { Db, Eth, Parity, Net, Personal, Shh, Signer, Trace, Web3 } from './rpc';
import Subscriptions from './subscriptions';
import util from './util';
import { isFunction } from './util/types';
import { LocalAccountsMiddleware } from './local';

export default class Api extends EventEmitter {
  constructor (provider, allowSubscriptions = true) {
    super();

    if (!provider || (!isFunction(provider.send) && !isFunction(provider.execute))) {
      throw new Error('Api needs provider with send() function');
    }

    if (!isFunction(provider.send)) {
      console.warn(new Error('deprecated: Api needs provider with send() function, old-style Transport found instead'));
    }

    this._provider = new PromiseProvider(provider);

    this._db = new Db(this._provider);
    this._eth = new Eth(this._provider);
    this._net = new Net(this._provider);
    this._parity = new Parity(this._provider);
    this._personal = new Personal(this._provider);
    this._shh = new Shh(this._provider);
    this._signer = new Signer(this._provider);
    this._trace = new Trace(this._provider);
    this._web3 = new Web3(this._provider);

    if (allowSubscriptions) {
      this._subscriptions = new Subscriptions(this);
    }

    // Doing a request here in test env would cause an error
    if (process.env.NODE_ENV !== 'test') {
      const middleware = this.parity
        .nodeKind()
        .then((nodeKind) => {
          if (nodeKind.availability === 'public') {
            return LocalAccountsMiddleware;
          }

          return null;
        })
        .catch(() => null);

      provider.addMiddleware(middleware);
    }
  }

  get db () {
    return this._db;
  }

  get eth () {
    return this._eth;
  }

  get parity () {
    return this._parity;
  }

  get net () {
    return this._net;
  }

  get personal () {
    return this._personal;
  }

  get provider () {
    return this._provider.provider;
  }

  get shh () {
    return this._shh;
  }

  get signer () {
    return this._signer;
  }

  get trace () {
    return this._trace;
  }

  get transport () {
    return this.provider;
  }

  get web3 () {
    return this._web3;
  }

  get util () {
    return util;
  }

  newContract (abi, address) {
    return new Contract(this, abi).at(address);
  }

  subscribe (subscriptionName, callback) {
    if (!this._subscriptions) {
      return Promise.resolve(1);
    }

    return this._subscriptions.subscribe(subscriptionName, callback);
  }

  unsubscribe (subscriptionId) {
    if (!this._subscriptions) {
      return Promise.resolve(true);
    }

    return this._subscriptions.unsubscribe(subscriptionId);
  }

  pollMethod (method, input, validate) {
    const [_group, endpoint] = method.split('_');
    const group = `_${_group}`;

    return new Promise((resolve, reject) => {
      const timeout = () => {
        this[group][endpoint](input)
          .then((result) => {
            if (validate ? validate(result) : result) {
              resolve(result);
            } else {
              setTimeout(timeout, 500);
            }
          })
          .catch((error) => {
            // Don't print if the request is rejected: that's ok
            if (error.type !== 'REQUEST_REJECTED') {
              console.error('pollMethod', error);
            }

            reject(error);
          });
      };

      timeout();
    });
  }

  static util = util

  static Provider = {
    Http: HttpProvider,
    PostMessage: PostMessageProvider,
    WsSecure: WsSecureProvider
  }

  // NOTE: kept for backwards compatibility
  static Transport = {
    Http: HttpTransport,
    WsSecure: WsSecureTransport
  }
}
