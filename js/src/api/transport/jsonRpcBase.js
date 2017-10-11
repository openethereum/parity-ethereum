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

import { Logging } from '../subscriptions';
import logger from './logger';

const LOGGER_ENABLED = process.env.NODE_ENV !== 'production';

export default class JsonRpcBase extends EventEmitter {
  constructor () {
    super();

    this._id = 1;
    this._debug = false;
    this._connected = false;
    this._middlewareList = Promise.resolve([]);
  }

  get ready () {
    return this._middlewareList.then(() => true);
  }

  encode (method, params) {
    const json = JSON.stringify({
      jsonrpc: '2.0',
      method: method,
      params: params,
      id: this._id++
    });

    return json;
  }

  addMiddleware (Middleware) {
    this._middlewareList = Promise
      .all([
        Middleware,
        this._middlewareList
      ])
      .then(([Middleware, middlewareList]) => {
        // Do nothing if `handlerPromise` resolves to a null-y value.
        if (Middleware == null) {
          return middlewareList;
        }

        // don't mutate the original array
        return middlewareList.concat([new Middleware(this)]);
      });
  }

  _wrapSuccessResult (result) {
    return {
      id: this._id,
      jsonrpc: '2.0',
      result
    };
  }

  _wrapErrorResult (error) {
    return {
      id: this._id,
      jsonrpc: '2.0',
      error: {
        code: error.code,
        message: error.text
      }
    };
  }

  execute (method, ...params) {
    let start;
    let logId;

    if (LOGGER_ENABLED) {
      start = Date.now();
      logId = logger.log({ method, params });
    }

    return this._middlewareList.then((middlewareList) => {
      for (const middleware of middlewareList) {
        const res = middleware.handle(method, params);

        if (res != null) {
          return Promise
            .resolve(res)
            .then((res) => {
              const result = this._wrapSuccessResult(res);
              const json = this.encode(method, params);

              Logging.send(method, params, { json, result });

              return res;
            });
        }
      }

      const result = this._execute(method, params);

      if (!LOGGER_ENABLED) {
        return result;
      }

      return result
        .then((result) => {
          logger.set(logId, { result, time: Date.now() - start });

          return result;
        });
    });
  }

  _execute () {
    throw new Error('Missing implementation of JsonRpcBase#_execute');
  }

  _setConnected () {
    if (!this._connected) {
      this._connected = true;
      this.emit('open');
    }
  }

  _setDisconnected () {
    if (this._connected) {
      this._connected = false;
      this.emit('close');
    }
  }

  get id () {
    return this._id;
  }

  get isDebug () {
    return this._debug;
  }

  get isConnected () {
    return this._connected;
  }

  setDebug (flag) {
    this._debug = flag;
  }

  error (error) {
    if (this.isDebug) {
      console.error(error);
    }
  }

  log (log) {
    if (this.isDebug) {
      console.log(log);
    }
  }
}
