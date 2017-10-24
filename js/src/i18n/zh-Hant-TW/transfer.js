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
      hint: `交易附帶資料`, // the data to pass through with the transaction
      label: `交易資料`// transaction data
    }
  },
  buttons: {
    back: `返回`, // Back
    cancel: `取消`, // Cancel
    close: `關閉`, // Close
    next: `下一步`, // Next
    send: `傳送`// Send
  },
  details: {
    advanced: {
      label: `高階傳送選項`// advanced sending options
    },
    amount: {
      hint: `傳送數額`, // the amount to transfer to the recipient
      label: `傳送數額{tag}`// amount to transfer (in {tag})
    },
    fullBalance: {
      label: `所有的餘額`// full account balance
    },
    recipient: {
      hint: `收款人地址`, // the recipient address
      label: `收款人地址`// recipient address
    },
    sender: {
      hint: `傳送人地址`, // the sender address
      label: `傳送人地址`// sender address
    },
    total: {
      label: `傳送數額`// total transaction amount
    }
  },
  wallet: {
    confirmation: `這筆交易需要其他人的確認。`, // This transaction needs confirmation from other owners.
    operationHash: `操作雜湊`// operation hash
  },
  warning: {
    wallet_spent_limit: `這筆轉帳的數額超過了每日轉帳數額上限。此交易需要其他人的確認才可以傳送成功。`
    // This transaction value is above the remaining daily limit. It will need to be confirmed by other owners.
  }
};
