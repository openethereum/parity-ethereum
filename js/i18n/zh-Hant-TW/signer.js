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
  embedded: {
    noPending: `目前沒有待處理的確認等待你的處理`
    // There are currently no pending requests awaiting your confirmation
  },
  mainDetails: {
    editTx: `編輯條款/gas/gasPrice`, // Edit conditions/gas/gasPrice
    tooltips: {
      total1: `包括交易費的交易的總價值是{total} {type}`,
      // The value of the transaction including the mining fee is {total} {type}.
      total2: `(包括了交易費 {fee} {token})`,
      // (This includes a mining fee of {fee} {token})
      value1: `交易價值` // The value of the transaction.
    }
  },
  requestOrigin: {
    dapp: `來自dapp {url}`, // by a dapp at {url}
    ipc: `通過IPC會話`, // via IPC session
    rpc: `通過RPC{rpc}`, // via RPC {rpc}
    signerCurrent: `通過當前標籤頁`, // via current tab
    signerUI: `通過互動會話`, // via UI session
    unknownInterface: `通過未知互動`, // via unknown interface
    unknownRpc: `未明確的`, // unidentified
    unknownUrl: `未知URL` // unknown URL
  },
  requestsPage: {
    noPending: `沒有請求需要你的確認`, // There are no requests requiring your confirmation.
    pendingTitle: `待處理請求`, // Pending Requests
    queueTitle: `本地交易` // Local Transactions
  },
  sending: {
    hardware: {
      confirm: `請在你連線的硬體裝置上確認交易`, // Please confirm the transaction on your attached hardware device
      connect: `請在確認交易前連線你的硬體裝置` // Please attach your hardware device before confirming the transaction
    }
  },
  signRequest: {
    request: `一個簽名資料在請求你的帳號：`, // A request to sign data using your account:
    state: {
      confirmed: `通過`, // Confirmed
      rejected: `拒絕` // Rejected
    },
    unknownBinary: `（未知二進位制資料）`, // (Unknown binary data)
    warning: `警告：此操作的結果是不可逆的。請在確認資訊後再通過請求。`
    // WARNING: The consequences of doing this may be grave. Confirm the request only if you are sure.
  },
  title: `可信的Signer`, // Trusted Signer
  txPending: {
    buttons: {
      viewToggle: `檢視交易` // view transaction
    }
  },
  txPendingConfirm: {
    buttons: {
      confirmBusy: `通過中...`, // Confirming...
      confirmRequest: `通過請求` // Confirm Request
    },
    errors: {
      invalidWallet: `所提供的錢包檔案不可用` // Given wallet file is invalid.
    },
    password: {
      decrypt: {
        hint: `解金鑰匙`, // decrypt the key
        label: `鑰匙密碼` // Key Password
      },
      unlock: {
        hint: `解鎖帳戶`, // unlock the account
        label: `帳戶密碼` // Account Password
      }
    },
    passwordHint: `（提示）{passwordHint}`, // (hint) {passwordHint}
    selectKey: {
      hint: `此帳戶的鑰匙檔案`, // The keyfile to use for this account
      label: `選擇本地鑰匙` // Select Local Key
    },
    tooltips: {
      password: `請為此帳戶提供密碼` // Please provide a password for this account
    }
  },
  txPendingForm: {
    changedMind: `我改主意了`, // I've changed my mind
    reject: `拒絕請求` // reject request
  },
  txPendingReject: {
    buttons: {
      reject: `拒絕請求` // Reject Request
    },
    info: `你確定要拒絕請求嗎？`, // Are you sure you want to reject request?
    undone: `此操作是不可逆的`  // This cannot be undone
  }
};
