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

import BigNumber from 'bignumber.js';
import store from 'store';

import { ERROR_CODES } from '~/api/transport/error';
import { trackRequest as trackRequestUtil, parseTransactionReceipt } from '~/util/tx';

const LS_REQUESTS_KEY = '_parity::requests';

class CachedRequests {
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

const cachedRequests = new CachedRequests();

export const init = (api) => (dispatch) => {
  api.on('request', (rawRequest) => {
    const { requestId, ...others } = rawRequest;
    const { from, to, value, data, gas, ...extras } = others;
    const transaction = {
      from,
      to,
      data,
      value,
      gas
    };
    const request = {
      requestId,
      transaction,
      ...extras
    };

    dispatch(watchRequest(request));
  });

  api.on('connected', () => {
    cachedRequests.load(api).then((requests) => {
      requests.forEach((request) => dispatch(watchRequest(request)));
    });
  });
};

export const watchRequest = (request) => (dispatch, getState) => {
  const { requestId, transaction, ...extras } = request;
  const requestData = {
    requestId,
    transaction,
    ...extras
  };

  // Convert value to BigNumber
  requestData.transaction.value = new BigNumber(requestData.transaction.value || 0);
  dispatch(setRequest(requestId, requestData));
  dispatch(trackRequest(requestId, requestData));
};

export const trackRequest = (requestId, requestData) => (dispatch, getState) => {
  const { api } = getState();
  const { transactionHash = null } = requestData;

  trackRequestUtil(api, { requestId, transactionHash }, (error, data) => {
    if (error) {
      console.error(error);
      return dispatch(setRequest(requestId, { error }));
    }

    // Hide the request after 6 mined blocks
    if (data.transactionReceipt) {
      const { transactionReceipt } = data;
      let blockSubscriptionId = -1;

      // If the request was a contract deployment,
      // then add the contract with the saved metadata to the account
      if (requestData.metadata && requestData.metadata.deployment) {
        const { metadata } = requestData;

        const options = {
          ...requestData.transaction,
          metadata
        };

        parseTransactionReceipt(api, options, data.transactionReceipt)
          .then((contractAddress) => {
            // No contract address given, might need some confirmations
            // from the wallet owners...
            if (!contractAddress || /^(0x)?0*$/.test(contractAddress)) {
              return false;
            }

            metadata.blockNumber = data.transactionReceipt
              ? data.transactionReceipt.blockNumber.toNumber()
              : null;

            const prevRequest = getState().requests[requestId];
            const nextTransaction = {
              ...prevRequest.transaction,
              creates: contractAddress
            };

            dispatch(setRequest(requestId, { transaction: nextTransaction }));
            return Promise.all([
              api.parity.setAccountName(contractAddress, metadata.name),
              api.parity.setAccountMeta(contractAddress, metadata)
            ]);
          })
          .catch((error) => {
            console.error(error);
          });
      }

      api
        .subscribe('eth_blockNumber', (error, blockNumber) => {
          if (error || !blockNumber) {
            return;
          }

          // Transaction included in `blockHeight` blocks
          const blockHeight = blockNumber.minus(transactionReceipt.blockNumber).plus(1);
          const nextData = { blockHeight };

          // Hide the transaction after 6 blocks
          if (blockHeight.gt(6)) {
            return dispatch(hideRequest(requestId));
          }

          return dispatch(setRequest(requestId, nextData, false));
        })
        .then((subId) => {
          blockSubscriptionId = subId;
          return dispatch(setRequest(requestId, { blockSubscriptionId }, false));
        });
    }

    return dispatch(setRequest(requestId, data));
  });
};

export const hideRequest = (requestId) => (dispatch, getState) => {
  const { api, requests } = getState();
  const request = requests[requestId];

  dispatch(setRequest(requestId, { show: false }));

  // Delete it if an error occured or if completed
  if (request.error || request.transactionReceipt) {
    // Wait for the animation to be done to delete the request
    setTimeout(() => {
      dispatch(deleteRequest(requestId));
    }, 1000);
  }

  // Unsubscribe to eth-blockNumber if subscribed
  if (request.blockSubscriptionId) {
    api.unsubscribe(request.blockSubscriptionId);
  }
};

export const setRequest = (requestId, requestData, autoSetShow = true) => {
  if (autoSetShow && requestData.show === undefined) {
    requestData.show = true;
  }

  cachedRequests.save(requestId, requestData);

  return {
    type: 'setRequest',
    requestId, requestData
  };
};

export const deleteRequest = (requestId) => {
  cachedRequests.remove(requestId);

  return {
    type: 'deleteRequest',
    requestId
  };
};
