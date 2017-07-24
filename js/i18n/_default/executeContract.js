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

export default {
  busy: {
    posted: `Your transaction has been posted to the network`,
    title: `The function execution is in progress`,
    waitAuth: `Waiting for authorization in the Parity Signer`
  },
  button: {
    cancel: `cancel`,
    done: `done`,
    next: `next`,
    post: `post transaction`,
    prev: `prev`
  },
  details: {
    address: {
      hint: `from account`,
      label: `the account to transact with`
    },
    advancedCheck: {
      label: `advanced sending options`
    },
    amount: {
      hint: `the amount to send to with the transaction`,
      label: `transaction value (in ETH)`
    },
    function: {
      hint: `the function to call on the contract`,
      label: `function to execute`
    }
  },
  rejected: {
    state: `You can safely close this window, the function execution will not occur.`,
    title: `The execution has been rejected`
  },
  steps: {
    advanced: `advanced options`,
    complete: `complete`,
    rejected: `rejected`,
    sending: `sending`,
    transfer: `function details`
  }
};
