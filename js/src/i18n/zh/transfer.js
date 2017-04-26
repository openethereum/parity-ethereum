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
      hint: `交易附带数据`, // the data to pass through with the transaction
      label: `交易数据`// transaction data
    }
  },
  buttons: {
    back: `返回`, // Back
    cancel: `取消`, // Cancel
    close: `关闭`, // Close
    next: `下一步`, // Next
    send: `发送`// Send
  },
  details: {
    advanced: {
      label: `高级发送选项`// advanced sending options
    },
    amount: {
      hint: `发送数额`, // the amount to transfer to the recipient
      label: `发送数额{tag}`// amount to transfer (in {tag})
    },
    fullBalance: {
      label: `所有的余额`// full account balance
    },
    recipient: {
      hint: `收款人地址`, // the recipient address
      label: `收款人地址`// recipient address
    },
    sender: {
      hint: `发送人地址`, // the sender address
      label: `发送人地址`// sender address
    },
    total: {
      label: `发送数额`// total transaction amount
    }
  },
  wallet: {
    confirmation: `这笔交易需要其他人的确认。`, // This transaction needs confirmation from other owners.
    operationHash: `操作哈希`// operation hash
  },
  warning: {
    wallet_spent_limit: `这笔转账的数额超过了每日转账数额上限。此交易需要其他人的确认才可以发送成功。`
    // This transaction value is above the remaining daily limit. It will need to be confirmed by other owners.
  }
};
