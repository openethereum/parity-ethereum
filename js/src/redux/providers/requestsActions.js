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

import { outTransaction } from '~/api/format/output';
import { trackRequest as trackRequestUtil, parseTransactionReceipt } from '~/util/tx';
import SavedRequests from '~/views/Application/Requests/savedRequests';

const savedRequests = new SavedRequests();

export const init = (api) => (dispatch) => {
  api.subscribe('signer_requestsToConfirm', (error, pending) => {
    if (error) {
      return;
    }

    const requests = pending
      .filter((p) => p.payload && p.payload.sendTransaction)
      .map((p) => ({
        requestId: '0x' + p.id.toString(16),
        transaction: p.payload.sendTransaction
      }));

    requests.forEach((request) => dispatch(watchRequest(request)));
  });

  api.once('connected', () => {
    savedRequests.load(api).then((requests) => {
      requests.forEach((request) => dispatch(watchRequest(request)));
    });
  });
};

export const watchRequest = (request) => (dispatch, getState) => {
  const { requestId } = request;

  // Convert value to BigNumber
  request.transaction = outTransaction(request.transaction);
  dispatch(setRequest(requestId, request));
  dispatch(trackRequest(requestId, request));
};

export const trackRequest = (requestId, { transactionHash = null } = {}) => (dispatch, getState) => {
  const { api } = getState();

  trackRequestUtil(api, { requestId, transactionHash }, (error, _data = {}) => {
    const data = { ..._data };

    if (error) {
      console.error(error);
      return dispatch(setRequest(requestId, { error }));
    }

    // Hide the request after 6 mined blocks
    if (data.transactionReceipt) {
      const { transactionReceipt } = data;
      const { requests } = getState();
      const requestData = requests[requestId];
      let blockSubscriptionId = -1;

      // Set the block height to 1 at the beginning (transaction mined,
      // thus one confirmation)
      data.blockHeight = new BigNumber(1);

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
    dispatch(setRequest(requestId, { blockSubscriptionId: null }, false));
  }
};

export const setRequest = (requestId, requestData, autoSetShow = true) => {
  if (autoSetShow && requestData.show === undefined) {
    requestData.show = true;
  }

  savedRequests.save(requestId, requestData);

  return {
    type: 'setRequest',
    requestId, requestData
  };
};

export const deleteRequest = (requestId) => {
  savedRequests.remove(requestId);

  return {
    type: 'deleteRequest',
    requestId
  };
};
