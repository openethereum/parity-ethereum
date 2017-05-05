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
  accounts: {
    button: {
      cancel: `取消`, // Cancel
      execute: `設定` // Set
    },
    empty: `此保險庫中沒有帳戶`, // There are no accounts in this vault
    title: `管理保險庫帳戶` // Manage Vault Accounts
  },
  button: {
    accounts: `帳戶`, // accounts
    add: `建立保險庫`, // create vault
    close: `關閉`, // close
    edit: `編輯`, // edit
    open: `開啟` // open
  },
  confirmClose: {
    info: `你即將關閉一個保險庫。所有與這個保險庫相關的帳戶在這個操作完成後都不再可見。如果想再見到關聯帳戶，請重新開啟保險庫。`,
    // You are about to close a vault. Any accounts associated with the vault won't be visible after this operation concludes. To view
    // the associated accounts, open the vault again.
    title: `關閉保險庫` // Close Vault
  },
  confirmOpen: {
    info: `你即將開啟一個保險庫。在確認了你的密碼之後，所有與這個保險庫關聯的帳戶都會可見。關閉保險庫會在介面中移除所有帳戶，直到保險庫被再次開啟。`,
    // You are about to open a vault. After confirming your password, all accounts associated with this vault will be visible. Closing
    // the vault will remove the accounts from view until the vault is opened again.
    password: {
      hint: `建立保險庫時設定的密碼`, // the password specified when creating the vault
      label: `保險庫密碼` // vault password
    },
    title: `開啟保險庫` // Open Vault
  },
  create: {
    button: {
      close: `關閉`, // close
      vault: `建立保險庫` // create valut
    },
    description: {
      hint: `該保險庫更詳細的描述` // an extended description for the vault
    },
    descriptions: {
      label: `（可選）描述` // (optional) description
    },
    hint: {
      hint: `（可選）一個幫助記憶密碼的提示`, // (optional) a hint to help with remembering the password
      label: `密碼提示` // password hint
    },
    name: {
      hint: `一個保險庫的名字`, // a name for the vault
      label: `保險庫名稱` // vault name
    },
    password: {
      hint: `一個高強度且獨一無二的密碼`, // a strong, unique password
      label: `密碼` // password
    },
    password2: {
      hint: `驗證你的密碼`, // verify your password
      label: `密碼（重複）` // password (repeat)
    },
    title: `建立一個新的保險庫` // Create a new vault
  },
  editMeta: {
    allowPassword: `更改保險庫密碼`, // Change vault password
    button: {
      close: `關閉`, // close
      save: `儲存` // save
    },
    currentPassword: {
      hint: `保險庫的原密碼`, // your current vault password
      label: `原密碼` // current password
    },
    description: {
      hint: `此保險庫的描述`, // the description for this vault
      label: `保險庫描述` // vault description
    },
    password: {
      hint: `一個高強度且獨一無二的密碼`, // a strong, unique password
      label: `新密碼` // new password
    },
    password2: {
      hint: `驗證你的新密碼`, // verify your new password
      label: `新密碼（重複）` // new password (repeat)
    },
    passwordHint: {
      hint: `此保險庫的密碼提示`, // your password hint for this vault
      label: `密碼提示` // password hint
    },
    title: `編輯保險庫元資料` // Edit Vault Metadata
  },
  empty: `目前沒有任何可顯示的保險庫`, // There are currently no vaults to display.
  selector: {
    noneAvailable: `目前沒有任何開啟、可選的保險庫。請在移動帳戶之前建立並開啟一個保險庫。`,
    // There are currently no vaults opened and available for selection. Create and open some first before attempting to select
    // a vault for an account move.
    title: `選擇帳戶保險庫` // Select Account Vault
  },
  title: `保險庫管理` // Vault Management
};
