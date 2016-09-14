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

import JsonRpcBase from '../jsonRpcBase';

/* global fetch */
export default class Http extends JsonRpcBase {
  constructor (url) {
    super();

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
    return fetch(this._url, this._encodeOptions(method, params))
      .then((response) => {
        if (response.status !== 200) {
          this.error(JSON.stringify({ status: response.status, statusText: response.statusText }));
          throw new Error(`${response.status}: ${response.statusText}`);
        }

        return response.json();
      })
      .then((result) => {
        if (result.error) {
          this.error(JSON.stringify(result));
          throw new Error(`${result.error.code}: ${result.error.message}`);
        }

        this.log(JSON.stringify(result));
        return result.result;
      });
  }
}
