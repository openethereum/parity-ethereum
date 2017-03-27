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
  buttons: {
    edit: `edit`,
    forget: `forget`,
    settings: `settings`,
    transfer: `transfer`
  },
  confirmations: {
    buttons: {
      confirmAs: `Confirm As...`,
      revokeAs: `Revoke As...`
    },
    none: `No transactions needs confirmation right now.`,
    tooltip: {
      confirmed: `Confirmed by {number}/{required} owners`
    }
  },
  details: {
    requiredOwners: `This wallet requires at least {owners} to validate any action (transactions, modifications).`,
    requiredOwnersNumber: `{number} {numberValue, plural, one {owner} other {owners}}`,
    spent: `{spent} has been spent today, out of {limit} set as the daily limit, which has been reset on {date}`,
    title: `Details`
  },
  title: `Wallet Management`,
  transactions: {
    none: `No transactions has been sent.`,
    title: `Transactions`
  }
};
