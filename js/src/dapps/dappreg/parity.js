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

const api = window.parent.secureApi;

function trackRequest (signerRequestId, statusCallback) {
  return api.pollMethod('parity_checkRequest', signerRequestId)
    .then((transactionHash) => {
      console.log('trackRequest', `received transaction hash ${transactionHash}`);
      statusCallback(null, { transactionHash });

      return api.pollMethod('eth_getTransactionReceipt', transactionHash, (receipt) => {
        if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
          return false;
        }

        return true;
      });
    })
    .then((transactionReceipt) => {
      console.log('trackRequest', 'received transaction receipt', transactionReceipt);
      statusCallback(null, { transactionReceipt });
    })
    .catch((error) => {
      statusCallback(error);
    });
}

export {
  api,
  trackRequest
};
