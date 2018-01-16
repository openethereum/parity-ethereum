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

import { stringify } from 'querystring';

export const isServerRunning = (isTestnet = false) => {
  const port = isTestnet ? 28443 : 18443;

  return fetch(`https://email-verification.parity.io:${port}/health`, {
    mode: 'cors',
    cache: 'no-store'
  })
    .then((res) => {
      return res.ok;
    })
    .catch(() => {
      return false;
    });
};

export const hasReceivedCode = (email, address, isTestnet = false) => {
  const port = isTestnet ? 28443 : 18443;
  const query = stringify({ email, address });

  return fetch(`https://email-verification.parity.io:${port}/?${query}`, {
    mode: 'cors',
    cache: 'no-store'
  })
    .then((res) => {
      return res.ok;
    })
    .catch(() => {
      return false; // todo: check for 404
    });
};

export const postToServer = (query, isTestnet = false) => {
  const port = isTestnet ? 28443 : 18443;

  query = stringify(query);

  return fetch(`https://email-verification.parity.io:${port}/?${query}`, {
    method: 'POST',
    mode: 'cors',
    cache: 'no-store'
  })
  .then((res) => {
    return res.json().then((data) => {
      if (res.ok) {
        return data.message;
      }
      throw new Error(data.message || 'unknown error');
    });
  });
};
