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
    phrase: {
      backedUp: `Type "I have written down the phrase" below to confirm it is backed up.`,
      backup: `Please back up the recovery phrase now. Make sure to keep it private and secure, it allows full and unlimited access to the account.`,
      backupConfirm: `Type your recovery phrase now.`,
      hint: `the account recovery phrase`,
      label: `owner recovery phrase`
    }
  },
  accountDetailsGeth: {
    imported: `You have completed the import of {number} addresses from the Geth keystore. These will now be available in your accounts list as a normal account, along with their associated balances on the network.`
  },
  button: {
    back: `Back`,
    cancel: `Cancel`,
    create: `Create`,
    done: `Done`,
    import: `Import`,
    next: `Next`,
    print: `Print Phrase`
  },
  creationType: {
    fromGeth: {
      description: `Import accounts from the Geth keystore with the original password`,
      label: `Geth keystore`
    },
    fromJSON: {
      description: `Import an industry-standard JSON keyfile with the original password`,
      label: `JSON file`
    },
    fromNew: {
      description: `Selecting your identity icon and specifying the password`,
      label: `New Account`
    },
    fromPresale: {
      description: `Import an Ethereum presale wallet file with the original password`,
      label: `Presale wallet`
    },
    fromQr: {
      description: `Attach an externally managed account via QR code`,
      label: `External Account`
    },
    fromRaw: {
      description: `Enter a previously created raw private key with a new password`,
      label: `Private key`
    },
    info: `Please select the type of account you want to create. Either create an account via name & password, or import it from a variety of existing sources. From here the wizard will guide you through the process of completing your account creation.`
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
    available: `There are currently {count} importable keys available from the Geth keystore which are not already available on your Parity instance. Select the accounts you wish to import and move to the next step to complete the import.`,
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
  newQr: {
    address: {
      hint: `the network address for the account`,
      label: `address`
    },
    description: {
      hint: `a description for the account`,
      label: `account description`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name`
    },
    summary: `Use the built-in machine camera to scan to QR code of the account you wish to attach as an external account. External accounts are signed on the external device.`
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
    passPhrase: {
      error: `enter a recovery phrase`
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
    warning: {
      emptyPhrase: `The recovery phrase is empty.
                This account can be recovered by anyone.`,
      shortPhrase: `The recovery phrase is less than 11 words.
                This account has not been generated by Parity and might be insecure.
                Proceed with caution.`,
      testnetEmptyPhrase: `The recovery phrase is empty.
                This account can be recovered by anyone.
                Proceed with caution.`
    },
    windowsKey: {
      label: `Key was created with Parity <1.4.5 on Windows`
    }
  },
  title: {
    accountInfo: `account information`,
    backupPhrase: `confirm recovery phrase`,
    createAccount: `create account`,
    createType: `creation type`,
    importAccount: `import account`,
    qr: `external account`,
    restoreAccount: `restore account`
  }
};
