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

const options = {
  method: 'GET',
  headers: {
    'Accept': 'application/json'
  }
};

export function call (module, action, _params, test) {
  const host = test ? 'testnet.etherscan.io' : 'api.etherscan.io';
  let params = '';

  if (_params) {
    Object.keys(_params).map((param) => {
      const value = _params[param];

      params = `${params}&${param}=${value}`;
    });
  }

  return fetch(`http://${host}/api?module=${module}&action=${action}${params}`, options)
    .then((response) => {
      if (response.status !== 200) {
        throw { code: response.status, message: response.statusText }; // eslint-disable-line
      }

      return response.json();
    })
    .then((result) => {
      if (result.message === 'NOTOK') {
        throw { code: -1, message: result.result }; // eslint-disable-line
      }

      return result.result;
    });
}
