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
    posted: `你的交易已被公佈至網路`, // Your transaction has been posted to the network
    title: `函式執行正在進行中`, // The function execution is in progress
    waitAuth: `正在等待Parity Signer授權` // Waiting for authorization in the Parity Signer
  },
  button: {
    cancel: `取消`, // cancel
    done: `完成`, // done
    next: `下一步`, // next
    post: `公佈交易`, // post transaction
    prev: `上一步` // prev
  },
  details: {
    address: {
      hint: `來自帳戶`, // from account
      label: `將要交易的帳戶` // the account to transact with
    },
    advancedCheck: {
      label: `高階傳送選項` // advanced sending options
    },
    amount: {
      hint: `此交易將會發送的數量`, // the amount to send to with the transaction
      label: `交易價值（ETH）` // transaction value (in ETH)
    },
    function: {
      hint: `此合約將會呼叫的函式`, // the function to call on the contract
      label: `將執行的函式` // function to execute
    }
  },
  rejected: {
    state: `你可以安全的關閉此視窗，函式將不會被執行。`, // You can safely close this window, the function execution will not occur.
    title: `執行失敗` // The execution has been rejected
  },
  steps: {
    advanced: `高階選項`, // advanced options
    complete: `完成`, // complete
    rejected: `失敗`, // rejected
    sending: `傳送中`, // sending
    transfer: `函式詳情` // function details
  }
};
