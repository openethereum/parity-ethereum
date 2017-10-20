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
    noPending: `目前没有待处理的确认等待你的处理`
    // There are currently no pending requests awaiting your confirmation
  },
  mainDetails: {
    editTx: `编辑条款/gas/gasPrice`, // Edit conditions/gas/gasPrice
    tooltips: {
      total1: `包括交易费的交易的总价值是{total} {type}`,
      // The value of the transaction including the mining fee is {total} {type}.
      total2: `(包括了交易费 {fee} {token})`,
      // (This includes a mining fee of {fee} {token})
      value1: `交易价值` // The value of the transaction.
    }
  },
  requestOrigin: {
    dapp: `来自dapp {url}`, // by a dapp at {url}
    ipc: `通过IPC会话`, // via IPC session
    rpc: `通过RPC{rpc}`, // via RPC {rpc}
    signerCurrent: `通过当前标签页`, // via current tab
    signerUI: `通过交互会话`, // via UI session
    unknownInterface: `通过未知交互`, // via unknown interface
    unknownRpc: `未明确的`, // unidentified
    unknownUrl: `未知URL` // unknown URL
  },
  requestsPage: {
    noPending: `没有请求需要你的确认`, // There are no requests requiring your confirmation.
    pendingTitle: `待处理请求`, // Pending Requests
    queueTitle: `本地交易` // Local Transactions
  },
  sending: {
    hardware: {
      confirm: `请在你连接的硬件设备上确认交易`, // Please confirm the transaction on your attached hardware device
      connect: `请在确认交易前连接你的硬件设备` // Please attach your hardware device before confirming the transaction
    }
  },
  signRequest: {
    request: `一个签名数据在请求你的账号：`, // A request to sign data using your account:
    state: {
      confirmed: `通过`, // Confirmed
      rejected: `拒绝` // Rejected
    },
    unknownBinary: `（未知二进制数据）`, // (Unknown binary data)
    warning: `警告：此操作的结果是不可逆的。请在确认信息后再通过请求。`
    // WARNING: The consequences of doing this may be grave. Confirm the request only if you are sure.
  },
  title: `可信的Signer`, // Trusted Signer
  txPending: {
    buttons: {
      viewToggle: `查看交易` // view transaction
    }
  },
  txPendingConfirm: {
    buttons: {
      confirmBusy: `通过中...`, // Confirming...
      confirmRequest: `通过请求` // Confirm Request
    },
    errors: {
      invalidWallet: `所提供的钱包文件不可用` // Given wallet file is invalid.
    },
    password: {
      decrypt: {
        hint: `解密钥匙`, // decrypt the key
        label: `钥匙密码` // Key Password
      },
      unlock: {
        hint: `解锁账户`, // unlock the account
        label: `账户密码` // Account Password
      }
    },
    passwordHint: `（提示）{passwordHint}`, // (hint) {passwordHint}
    selectKey: {
      hint: `此账户的钥匙文件`, // The keyfile to use for this account
      label: `选择本地钥匙` // Select Local Key
    },
    tooltips: {
      password: `请为此账户提供密码` // Please provide a password for this account
    }
  },
  txPendingForm: {
    changedMind: `我改主意了`, // I've changed my mind
    reject: `拒绝请求` // reject request
  },
  txPendingReject: {
    buttons: {
      reject: `拒绝请求` // Reject Request
    },
    info: `你确定要拒绝请求吗？`, // Are you sure you want to reject request?
    undone: `此操作是不可逆的`  // This cannot be undone
  }
};
