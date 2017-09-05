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
  addOwner: {
    title: `添加持有人` // Add Owner
  },
  buttons: {
    cancel: `取消`, // Cancel
    close: `关闭`, // Close
    next: `下一个`, // Next
    send: `发送`, // Send
    sending: `正在发送...` // Sending...
  },
  changes: {
    modificationString: `为了保证你做的修改会被执行，
                        其他的账户持有人也需要发送相同的修改。
                        他们可以通过粘贴以下字符串来简单的完成更改：`,
                        // For your modifications to be taken into account,
              // other owners have to send the same modifications. They can paste
              // this string to make it easier:
    none: `钱包设置没有发生任何更改。`, // No modifications have been made to the Wallet settings.
    overview: `你将会造成以下更改` // You are about to make the following modifications
  },
  edit: {
    message: `为了编辑这个合约的设置，至少{owners, number} 
              {owners, plural, one {owner } other {owners }}必须发送完全相同的修改
              你可以将字符串化的修改粘贴在这里。`
                  // In order to edit this contract's settings, at
                  // least {owners, number} {owners, plural, one {owner } other {owners }} have to
                  // send the very same modifications. You can paste a stringified version
                  // of the modifications here.
  },
  modifications: {
    daylimit: {
      hint: `不需要确认即可发送的ETH数量`, // amount of ETH spendable without confirmations
      label: `钱包每日限额` // wallet day limit
    },
    fromString: {
      label: `修改` // modifications
    },
    owners: {
      label: `其他钱包持有人` // other wallet owners
    },
    required: {
      hint: `确认交易所需的通过持有人人数`, // number of required owners to accept a transaction
      label: `所需持有人` // required owners
    },
    sender: {
      hint: `作为此持有人发送修改`, // send modifications as this owner
      label: `来自账户 (wallet owner)` // from account (wallet owner)
    }
  },
  ownersChange: {
    details: `从 {from} 至 {to}`, // from {from} to {to}
    title: `改变所需持有人`
  },
  rejected: `交易#{txid}已经被拒绝`, // The transaction #{txid} has been rejected
  removeOwner: {
    title: `移除持有人` // Remove Owner
  }
};
