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
      hint: `Het netwerk adres van het account`,
      label: `adres`
    },
    name: {
      hint: `Een beschrijvende naam van het account`,
      label: `account naam`
    },
    phrase: {
      hint: `De account herstel zin`,
      label: `Eigenaar's herstel zin (houd deze woorden veilig en prive want hiermee kun je volledige, ongelimiteerde toegang tot het account verkrijgen).`
    }
  },
  accountDetailsGeth: {
    imported: `Je hebt {number} adressen geïmporteerd uit de Geth keystore:`
  },
  button: {
    back: `Terug`,
    cancel: `Annuleer`,
    close: `Sluit`,
    create: `Aanmaken`,
    import: `Importeer`,
    next: `Volgende`,
    print: `Print herstel zin`
  },
  creationType: {
    fromGeth: {
      label: `Importeer accounts uit Geth keystore`
    },
    fromJSON: {
      label: `Importeer account uit een opgeslagen JSON file`
    },
    fromNew: {
      label: `Handmatig account aanmaken`
    },
    fromPhrase: {
      label: `Herstel account met een herstel zin`
    },
    fromPresale: {
      label: `Importeer account van een Ethereum voor-verkoop (pre-sale) wallet`
    },
    fromRaw: {
      label: `Importeer een kale prive sleutel (raw private key)`
    }
  },
  error: {
    invalidKey: `the raw key needs to be hex, 64 characters in length and contain the prefix "0x"`,
    noFile: `select a valid wallet file to import`,
    noKey: `you need to provide the raw private key`,
    noMatchPassword: `the supplied passwords does not match`,
    noName: `you need to specify a valid name for the account`
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
    accountInfo: `account informatie`,
    createAccount: `account aanmaken`,
    createType: `manier van aanmaken`,
    importWallet: `importeer wallet`
  }
};
