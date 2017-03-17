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

import { trackRequest } from '~/util/tx';

export const init = (api) => (dispatch) => {
  api.on('request', (request) => {
    dispatch(watchRequest(request));
  });
};

export const watchRequest = (request) => (dispatch, getState) => {
  const { api } = getState();
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

  trackRequest(api, requestId, (error, data) => {
    if (error) {
      return dispatch(setRequest(requestId, { error }));
    }

    // Hide the request after 6 mined blocks
    if (data.transactionReceipt) {
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
