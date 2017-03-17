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

import { trackRequest as trackRequestUtil } from '~/util/tx';

const LS_REQUESTS_KEY = '_parity::requests';

const getRequests = () => {
  return store.get(LS_REQUESTS_KEY) || {};
};

const setRequests = (requests = {}) => {
  return store.set(LS_REQUESTS_KEY, requests);
};

const loadRequests = () => (dispatch) => {
  const requests = getRequests();

  Object.values(requests).forEach((request) => {
    return dispatch(watchRequest(request));
  });
};

const saveRequest = (request) => {
  const requests = getRequests();

  requests[request.requestId] = {
    ...(requests[request.requestId] || {}),
    ...request
  };

  setRequests(requests);
};

const removeRequest = (requestId) => {
  const requests = getRequests();

  delete requests[requestId];
  setRequests(requests);
};

export const init = (api) => (dispatch) => {
  api.on('request', (request) => {
    dispatch(watchRequest(request));
  });

  // dispatch(loadRequests());
};

export const watchRequest = (request) => (dispatch, getState) => {
  const { requestId, ...others } = request;
  const { from, to, value, data } = others;
  const transaction = {
    from,
    to,
    data,
    value: new BigNumber(value || 0)
  };

  const requestData = {
    id: requestId,
    transaction
  };

  dispatch(setRequest(requestId, requestData));
  dispatch(trackRequest(requestId));

  saveRequest(request);
};

export const trackRequest = (requestId) => (dispatch, getState) => {
  const { api } = getState();

  trackRequestUtil(api, requestId, (error, data) => {
    if (error) {
      return dispatch(setRequest(requestId, { error }));
    }

    // Hide the request after 6 mined blocks
    if (data.transactionReceipt) {
      // Remove request from the localstorage
      removeRequest(requestId);

      const { transactionReceipt } = data;
      let blockSubscriptionId = -1;

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

  return {
    type: 'setRequest',
    requestId, requestData
  };
};

export const deleteRequest = (requestId) => {
  return {
    type: 'deleteRequest',
    requestId
  };
};
