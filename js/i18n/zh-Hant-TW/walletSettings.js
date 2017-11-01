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
    title: `新增持有人` // Add Owner
  },
  buttons: {
    cancel: `取消`, // Cancel
    close: `關閉`, // Close
    next: `下一個`, // Next
    send: `傳送`, // Send
    sending: `正在傳送...` // Sending...
  },
  changes: {
    modificationString: `為了保證你做的修改會被執行，
                        其他的帳戶持有人也需要傳送相同的修改。
                        他們可以通過貼上以下字串來簡單的完成更改：`,
                        // For your modifications to be taken into account,
              // other owners have to send the same modifications. They can paste
              // this string to make it easier:
    none: `錢包設定沒有發生任何更改。`, // No modifications have been made to the Wallet settings.
    overview: `你將會造成以下更改` // You are about to make the following modifications
  },
  edit: {
    message: `為了編輯這個合約的設定，至少{owners, number} 
              {owners, plural, one {owner } other {owners }}必須傳送完全相同的修改
              你可以將字串化的修改貼上在這裡。`
                  // In order to edit this contract's settings, at
                  // least {owners, number} {owners, plural, one {owner } other {owners }} have to
                  // send the very same modifications. You can paste a stringified version
                  // of the modifications here.
  },
  modifications: {
    daylimit: {
      hint: `不需要確認即可傳送的ETH數量`, // amount of ETH spendable without confirmations
      label: `錢包每日限額` // wallet day limit
    },
    fromString: {
      label: `修改` // modifications
    },
    owners: {
      label: `其他錢包持有人` // other wallet owners
    },
    required: {
      hint: `確認交易所需的通過持有人人數`, // number of required owners to accept a transaction
      label: `所需持有人` // required owners
    },
    sender: {
      hint: `作為此持有人傳送修改`, // send modifications as this owner
      label: `來自帳戶 (wallet owner)` // from account (wallet owner)
    }
  },
  ownersChange: {
    details: `從 {from} 至 {to}`, // from {from} to {to}
    title: `改變所需持有人`
  },
  rejected: `交易#{txid}已經被拒絕`, // The transaction #{txid} has been rejected
  removeOwner: {
    title: `移除持有人` // Remove Owner
  }
};
