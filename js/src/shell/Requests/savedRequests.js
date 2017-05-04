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

import store from 'store';

import { ERROR_CODES } from '@parity/api/transport/error';

export const LS_REQUESTS_KEY = '_parity::requests';

export default class SavedRequests {
  network = null;

  /**
   * Load the network version, and then the related requests
   */
  load (api) {
    return api.net.version()
      .then((network) => {
        this.network = network;
        return this.loadRequests(api);
      })
      .catch((error) => {
        console.error(error);
        return [];
      });
  }

  /**
   * Load the requests of the current network
   */
  loadRequests (api) {
    const requests = this._get();
    const promises = Object.values(requests).map((request) => {
      const { requestId, transactionHash } = request;

      // The request hasn't been signed yet
      if (transactionHash) {
        return request;
      }

      return this._requestExists(api, requestId)
        .then((exists) => {
          if (!exists) {
            return null;
          }

          return request;
        })
        .catch(() => {
          this.remove(requestId);
        });
    });

    return Promise.all(promises).then((requests) => requests.filter((request) => request));
  }

  save (requestId, requestData) {
    const requests = this._get();

    requests[requestId] = {
      ...(requests[requestId] || {}),
      ...requestData
    };

    this._set(requests);
  }

  remove (requestId) {
    const requests = this._get();

    delete requests[requestId];
    this._set(requests);
  }

  _get () {
    const allRequests = store.get(LS_REQUESTS_KEY) || {};

    return allRequests[this.network] || {};
  }

  _set (requests = {}) {
    const allRequests = store.get(LS_REQUESTS_KEY) || {};

    if (Object.keys(requests).length > 0) {
      allRequests[this.network] = requests;
    } else {
      delete allRequests[this.network];
    }

    return store.set(LS_REQUESTS_KEY, allRequests);
  }

  _requestExists (api, requestId) {
    return api.parity
      .checkRequest(requestId)
      .then(() => true)
      .catch((error) => {
        if (error.code === ERROR_CODES.REQUEST_NOT_FOUND) {
          return false;
        }

        throw error;
      });
  }
}
