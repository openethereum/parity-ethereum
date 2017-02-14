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
  details: {
    address: {
      label: `the account to transact with`,
      hint: `from account`
    },
    amount: {
      hint: `the amount to send to with the transaction`,
      label: `transaction value (in ETH)`
    },
    advancedCheck: {
      label: `advanced sending options`
    },
    function: {
      hint: `the function to call on the contract`,
      label: `function to execute`
    }
  },
  steps: {
    transfer: `function details`,
    sending: `sending`,
    complete: `complete`,
    advanced: `advanced options`,
    rejected: `rejected`
  },
  button: {
    cancel: `cancel`,
    post: `post transaction`,
    next: `next`,
    prev: `prev`,
    done: `done`
  },
  rejected: {
    title: `The execution has been rejected`,
    state: `You can safely close this window, the function execution will not occur.`
  },
  busy: {
    title: `The function execution is in progress`,
    waitAuth: `Waiting for authorization in the Parity Signer`,
    posted: `Your transaction has been posted to the network`
  }
};
