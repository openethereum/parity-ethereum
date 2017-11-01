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

const ENDPOINT = 'https://cors.shapeshift.io';

function call (method, options) {
  return fetch(`${ENDPOINT}/${method}`, options)
    .then((response) => {
      if (!response.ok) {
        throw new Error(response.statusText);
      }

      return response.json();
    })
    .then((result) => {
      if (result.error) {
        throw new Error(result.error);
      }

      return result;
    });
}

export default function (apiKey) {
  function get (method) {
    return call(method, {
      method: 'GET',
      headers: {
        'Accept': 'application/json'
      }
    });
  }

  function post (method, data) {
    const params = Object.assign({}, { apiKey }, data);
    const body = JSON.stringify(params);

    return call(method, {
      method: 'POST',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'Content-Length': body.length
      },
      body
    });
  }

  return {
    ENDPOINT,
    get,
    post
  };
}
