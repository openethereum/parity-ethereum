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
      cancel: `Cancel`,
      execute: `Set`
    },
    empty: `There are no accounts in this vault`,
    title: `Manage Vault Accounts`
  },
  button: {
    accounts: `accounts`,
    add: `create vault`,
    close: `close`,
    edit: `edit`,
    open: `open`
  },
  confirmClose: {
    info: `You are about to close a vault. Any accounts associated with the vault won't be visible after this operation concludes. To view the associated accounts, open the vault again.`,
    title: `Close Vault`
  },
  confirmOpen: {
    info: `You are about to open a vault. After confirming your password, all accounts associated with this vault will be visible. Closing the vault will remove the accounts from view until the vault is opened again.`,
    password: {
      hint: `the password specified when creating the vault`,
      label: `vault password`
    },
    title: `Open Vault`
  },
  create: {
    button: {
      close: `close`,
      vault: `create vault`
    },
    description: {
      hint: `an extended description for the vault`
    },
    descriptions: {
      label: `(optional) description`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    name: {
      hint: `a name for the vault`,
      label: `vault name`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password`
    },
    password2: {
      hint: `verify your password`,
      label: `password (repeat)`
    },
    title: `Create a new vault`
  },
  editMeta: {
    allowPassword: `Change vault password`,
    button: {
      close: `close`,
      save: `save`
    },
    currentPassword: {
      hint: `your current vault password`,
      label: `current password`
    },
    description: {
      hint: `the description for this vault`,
      label: `vault description`
    },
    password: {
      hint: `a strong, unique password`,
      label: `new password`
    },
    password2: {
      hint: `verify your new password`,
      label: `new password (repeat)`
    },
    passwordHint: {
      hint: `your password hint for this vault`,
      label: `password hint`
    },
    title: `Edit Vault Metadata`
  },
  empty: `There are currently no vaults to display.`,
  selector: {
    noneAvailable: `There are currently no vaults opened and available for selection. Create and open some first before attempting to select a vault for an account move.`,
    title: `Select Account Vault`
  },
  title: `Vault Management`
};
