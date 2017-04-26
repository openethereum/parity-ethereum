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
      execute: `设定` // Set
    },
    empty: `此保险库中没有账户`, // There are no accounts in this vault
    title: `管理保险库账户` // Manage Vault Accounts
  },
  button: {
    accounts: `账户`, // accounts
    add: `创建保险库`, // create vault
    close: `关闭`, // close
    edit: `编辑`, // edit
    open: `打开` // open
  },
  confirmClose: {
    info: `你即将关闭一个保险库。所有与这个保险库相关的账户在这个操作完成后都不再可见。如果想再见到关联账户，请重新打开保险库。`,
    // You are about to close a vault. Any accounts associated with the vault won't be visible after this operation concludes. To view
    // the associated accounts, open the vault again.
    title: `关闭保险库` // Close Vault
  },
  confirmOpen: {
    info: `你即将打开一个保险库。在确认了你的密码之后，所有与这个保险库关联的账户都会可见。关闭保险库会在界面中移除所有账户，直到保险库被再次打开。`,
    // You are about to open a vault. After confirming your password, all accounts associated with this vault will be visible. Closing
    // the vault will remove the accounts from view until the vault is opened again.
    password: {
      hint: `创建保险库时设置的密码`, // the password specified when creating the vault
      label: `保险库密码` // vault password
    },
    title: `打开保险库` // Open Vault
  },
  create: {
    button: {
      close: `关闭`, // close
      vault: `创建保险库` // create valut
    },
    description: {
      hint: `该保险库更详细的描述` // an extended description for the vault
    },
    descriptions: {
      label: `（可选）描述` // (optional) description
    },
    hint: {
      hint: `（可选）一个帮助记忆密码的提示`, // (optional) a hint to help with remembering the password
      label: `密码提示` // password hint
    },
    name: {
      hint: `一个保险库的名字`, // a name for the vault
      label: `保险库名称` // vault name
    },
    password: {
      hint: `一个高强度且独一无二的密码`, // a strong, unique password
      label: `密码` // password
    },
    password2: {
      hint: `验证你的密码`, // verify your password
      label: `密码（重复）` // password (repeat)
    },
    title: `创建一个新的保险库` // Create a new vault
  },
  editMeta: {
    allowPassword: `更改保险库密码`, // Change vault password
    button: {
      close: `关闭`, // close
      save: `保存` // save
    },
    currentPassword: {
      hint: `保险库的原密码`, // your current vault password
      label: `原密码` // current password
    },
    description: {
      hint: `此保险库的描述`, // the description for this vault
      label: `保险库描述` // vault description
    },
    password: {
      hint: `一个高强度且独一无二的密码`, // a strong, unique password
      label: `新密码` // new password
    },
    password2: {
      hint: `验证你的新密码`, // verify your new password
      label: `新密码（重复）` // new password (repeat)
    },
    passwordHint: {
      hint: `此保险库的密码提示`, // your password hint for this vault
      label: `密码提示` // password hint
    },
    title: `编辑保险库元数据` // Edit Vault Metadata
  },
  empty: `目前没有任何可显示的保险库`, // There are currently no vaults to display.
  selector: {
    noneAvailable: `目前没有任何打开、可选的保险库。请在移动账户之前创建并打开一个保险库。`,
    // There are currently no vaults opened and available for selection. Create and open some first before attempting to select
    // a vault for an account move.
    title: `选择账户保险库` // Select Account Vault
  },
  title: `保险库管理` // Vault Management
};
