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

import { ERROR_CODES } from '~/api/transport/error';

export const LS_REQUESTS_KEY = '_parity::requests';

export default class SavedRequests {
  load (api) {
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
    return store.get(LS_REQUESTS_KEY) || {};
  }

  _set (requests = {}) {
    return store.set(LS_REQUESTS_KEY, requests);
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
