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
  advanced: {
    data: {
      hint: `the data to pass through with the transaction`,
      label: `transaction data`
    }
  },
  buttons: {
    back: `Back`,
    cancel: `Cancel`,
    close: `Close`,
    next: `Next`,
    send: `Send`
  },
  details: {
    advanced: {
      label: `advanced sending options`
    },
    amount: {
      hint: `the amount to transfer to the recipient`,
      label: `amount to transfer (in {tag})`
    },
    fullBalance: {
      label: `full account balance`
    },
    recipient: {
      hint: `the recipient address`,
      label: `recipient address`
    },
    sender: {
      hint: `the sender address`,
      label: `sender address`
    },
    total: {
      label: `total transaction amount`
    }
  },
  wallet: {
    confirmation: `This transaction needs confirmation from other owners.`,
    operationHash: `operation hash`
  },
  warning: {
    wallet_spent_limit: `This transaction value is above the remaining daily limit. It will need to be confirmed by other owners.`
  }
};
