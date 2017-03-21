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
  accountDetails: {
    address: {
      hint: `the network address for the account`,
      label: `address`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    phrase: {
      hint: `the account recovery phrase`,
      label: `owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)`
    }
  },
  accountDetailsGeth: {
    imported: `You have imported {number} addresses from the Geth keystore:`
  },
  button: {
    back: `Back`,
    cancel: `Cancel`,
    close: `Close`,
    create: `Create`,
    import: `Import`,
    next: `Next`,
    print: `Print Phrase`
  },
  creationType: {
    fromGeth: {
      label: `Import accounts from Geth keystore`
    },
    fromJSON: {
      label: `Import account from a backup JSON file`
    },
    fromNew: {
      label: `Create new account manually`
    },
    fromPhrase: {
      label: `Recover account from recovery phrase`
    },
    fromPresale: {
      label: `Import account from an Ethereum pre-sale wallet`
    },
    fromRaw: {
      label: `Import raw private key`
    }
  },
  newAccount: {
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password`
    },
    password2: {
      hint: `verify your password`,
      label: `password (repeat)`
    }
  },
  newGeth: {
    noKeys: `There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance`
  },
  newImport: {
    file: {
      hint: `the wallet file for import`,
      label: `wallet file`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    password: {
      hint: `the password to unlock the wallet`,
      label: `password`
    }
  },
  rawKey: {
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password`
    },
    password2: {
      hint: `verify your password`,
      label: `password (repeat)`
    },
    private: {
      hint: `the raw hex encoded private key`,
      label: `private key`
    }
  },
  recoveryPhrase: {
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password`
    },
    password2: {
      hint: `verify your password`,
      label: `password (repeat)`
    },
    phrase: {
      hint: `the account recovery phrase`,
      label: `account recovery phrase`
    },
    windowsKey: {
      label: `Key was created with Parity <1.4.5 on Windows`
    }
  },
  title: {
    accountInfo: `account information`,
    createAccount: `create account`,
    createType: `creation type`,
    importWallet: `import wallet`
  }
};
