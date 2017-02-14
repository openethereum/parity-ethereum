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
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    address: {
      hint: `the network address for the account`,
      label: `address`
    },
    phrase: {
      hint: `the account recovery phrase`,
      label: `owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)`
    }
  },
  accountDetailsGeth: {
    imported: `You have imported {number} addresses from the Geth keystore:`
  },
  title: {
    createType: `creation type`,
    createAccount: `create account`,
    accountInfo: `account information`,
    importWallet: `import wallet`
  },
  button: {
    cancel: `Cancel`,
    next: `Next`,
    back: `Back`,
    create: `Create`,
    import: `Import`,
    print: `Print Phrase`,
    close: `Close`
  },
  creationType: {
    fromNew: {
      label: `Create new account manually`
    },
    fromPhrase: {
      label: `Recover account from recovery phrase`
    },
    fromGeth: {
      label: `Import accounts from Geth keystore`
    },
    fromJSON: {
      label: `Import account from a backup JSON file`
    },
    fromPresale: {
      label: `Import account from an Ethereum pre-sale wallet`
    },
    fromRaw: {
      label: `Import raw private key`
    }
  },
  error: {
    noFile: `select a valid wallet file to import`,
    noKey: `you need to provide the raw private key`,
    noMatchPassword: `the supplied passwords does not match`,
    noName: `you need to specify a valid name for the account`,
    invalidKey: `the raw key needs to be hex, 64 characters in length and contain the prefix "0x"`
  },
  newAccount: {
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
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
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    password: {
      hint: `the password to unlock the wallet`,
      label: `password`
    },
    file: {
      hint: `the wallet file for import`,
      label: `wallet file`
    }
  },
  rawKey: {
    private: {
      hint: `the raw hex encoded private key`,
      label: `private key`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
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
  recoveryPhrase: {
    phrase: {
      hint: `the account recovery phrase`,
      label: `account recovery phrase`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password`
    },
    password2: {
      hint: `verify your password`,
      label: `password (repeat)`
    },
    windowsKey: {
      label: `Key was created with Parity <1.4.5 on Windows`
    }
  }
};
