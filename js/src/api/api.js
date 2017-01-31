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

import { Http, Ws } from './transport';
import Contract from './contract';

import { Db, Eth, Parity, Net, Personal, Shh, Signer, Trace, Web3 } from './rpc';
import Subscriptions from './subscriptions';
import util from './util';
import { isFunction } from './util/types';

export default class Api extends EventEmitter {
  constructor (transport) {
    super();

    if (!transport || !isFunction(transport.execute)) {
      throw new Error('EthApi needs transport with execute() function defined');
    }

    this._transport = transport;

    this._db = new Db(transport);
    this._eth = new Eth(transport);
    this._net = new Net(transport);
    this._parity = new Parity(transport);
    this._personal = new Personal(transport);
    this._shh = new Shh(transport);
    this._signer = new Signer(transport);
    this._trace = new Trace(transport);
    this._web3 = new Web3(transport);

    this._subscriptions = new Subscriptions(this);
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
    return this._transport;
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
    return this._subscriptions.subscribe(subscriptionName, callback);
  }

  unsubscribe (subscriptionId) {
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

  static Transport = {
    Http: Http,
    Ws: Ws
  }
}
