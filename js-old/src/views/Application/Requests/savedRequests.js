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

export const LS_REQUESTS_KEY = '_parity::requests';

export default class SavedRequests {
  load (api) {
    return this.loadRequests(api)
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

    return api.parity.localTransactions()
      .then((localTransactions) => {
        const promises = Object.values(requests).map((request) => {
          const { requestId, transactionHash } = request;

          if (transactionHash) {
            // The transaction might be from an other
            // chain
            if (!localTransactions[transactionHash]) {
              this.remove(requestId);
              return null;
            }

            return request;
          }

          // The request hasn't been signed yet
          return this._requestExists(api, requestId)
            .then((exists) => {
              if (!exists) {
                this.remove(requestId);
                return null;
              }

              return request;
            });
        });

        return Promise.all(promises);
      })
      .then((requests) => requests.filter((request) => request));
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
    return store.get(LS_REQUESTS_KEY) || {};
  }

  _set (requests = {}) {
    if (Object.keys(requests).length > 0) {
      return store.set(LS_REQUESTS_KEY, requests);
    }

    return store.remove(LS_REQUESTS_KEY);
  }

  _requestExists (api, requestId) {
    return api.parity
      .checkRequest(requestId)
      .then(() => true)
      .catch(() => {
        return false;
      });
  }
}
