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

import { Logging } from '../../subscriptions';
import JsonRpcBase from '../jsonRpcBase';

/* global fetch */
export default class Http extends JsonRpcBase {
  constructor (url) {
    super();

    this._connected = true;
    this._url = url;
  }

  _encodeOptions (method, params) {
    const json = this.encode(method, params);

    this.log(json);

    return {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'Content-Length': json.length
      },
      body: json
    };
  }

  execute (method, ...params) {
    const request = this._encodeOptions(method, params);

    return fetch(this._url, request)
      .catch((error) => {
        this._connected = false;
        throw error;
      })
      .then((response) => {
        this._connected = true;

        if (response.status !== 200) {
          this._connected = false;
          this.error(JSON.stringify({ status: response.status, statusText: response.statusText }));
          throw new Error(`${response.status}: ${response.statusText}`);
        }

        return response.json();
      })
      .then((response) => {
        Logging.send(method, params, { request, response });

        if (response.error) {
          this.error(JSON.stringify(response));
          throw new Error(`${response.error.code}: ${response.error.message}`);
        }

        this.log(JSON.stringify(response));
        return response.result;
      });
  }
}
