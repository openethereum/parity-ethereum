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
    edit: `編輯`, // edit
    forget: `forget`,
    settings: `設定`, // settings
    transfer: `轉帳`// transfer
  },
  confirmations: {
    buttons: {
      confirmAs: `確定為……`, // Confirm As...
      revokeAs: `撤回為……`// Revoke As...
    },
    none: `現在沒有交易需要確認。`, // No transactions needs confirmation right now.
    tooltip: {
      confirmed: `被{number}/{required}所有人確認`// Confirmed by {number}/{required} owners
    }
  },
  details: {
    requiredOwners: `這個錢包需要至少{owners}所有人驗證所有的操作（交易，修改）`,
    // This wallet requires at least {owners} to validate any action (transactions, modifications).
    requiredOwnersNumber: `{number} {numberValue, plural, one {owner} other {owners}}`,
    spent: `{spent} has been spent today, out of {limit} set as the daily limit, which has been reset on {date}`,
    title: `細節`// Details
  },
  title: `錢包管理`, // Wallet Management
  transactions: {
    none: `沒有交易被髮送。`, // No transactions has been sent.
    title: `交易`// Transactions
  }
};
