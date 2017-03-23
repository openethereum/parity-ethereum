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
      label: `address 地址`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name账户名`
    },
    phrase: {
      hint: `the account recovery phrase账户恢复词组`,
      label: `owner recovery phrase (keep private and secure, it allows full and unlimited access to the account)`
    }
  },
  accountDetailsGeth: {
    imported: `You have imported {number} addresses from the Geth keystore:你已经从Geth keystore导入了`
  },
  button: {
    back: `Back返回`,
    cancel: `Cancel取消`,
    close: `Close关闭`,
    create: `Create创建`,
    import: `Import导入`,
    next: `Next下一步`,
    print: `Print Phrase打印词组`
  },
  creationType: {
    fromGeth: {
      label: `Import accounts from Geth keystore从Geth keystore导入账户`
    },
    fromJSON: {
      label: `Import account from a backup JSON file从JSON文件导入账户`
    },
    fromNew: {
      label: `Create new account manually手动创建新账户`
    },
    fromPhrase: {
      label: `Recover account from recovery phrase通过助记词恢复账户`
    },
    fromPresale: {
      label: `Import account from an Ethereum pre-sale wallet从以太坊预售钱包导入账户`
    },
    fromRaw: {
      label: `Import raw private key导入私钥`
    }
  },
  newAccount: {
    hint: {
      hint: `(optional) a hint to help with remembering the password`,
      label: `password hint密码提示`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name账户名`
    },
    password: {
      hint: `a strong, unique password足够强的密码`,
      label: `password密码`
    },
    password2: {
      hint: `verify your password确认你的密码`,
      label: `password (repeat)再次输入密码`
    }
  },
  newGeth: {
    noKeys: `There are currently no importable keys available from the Geth keystore, which are not already available on your Parity instance`
  },
  newImport: {
    file: {
      hint: `the wallet file for import要导入的钱包文件`,
      label: `wallet file钱包文件`
    },
    hint: {
      hint: `(optional) a hint to help with remembering the password（可选）帮助你记住密码的提示`,
      label: `password hint密码提示`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name账户名`
    },
    password: {
      hint: `the password to unlock the wallet输入密码，解锁钱包`,
      label: `password密码`
    }
  },
  rawKey: {
    hint: {
      hint: `(optional) a hint to help with remembering the password（可选）帮助你记住密码的提示`,
      label: `password hint密码提示`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name账户名`
    },
    password: {
      hint: `a strong, unique password足够强的密码`,
      label: `password密码`
    },
    password2: {
      hint: `verify your password确认密码`,
      label: `password (repeat)再次输入密码`
    },
    private: {
      hint: `the raw hex encoded private key`,
      label: `private key私钥`
    }
  },
  recoveryPhrase: {
    hint: {
      hint: `(optional) a hint to help with remembering the password（可选）帮助你记住密码的提示`,
      label: `password hint密码提示`
    },
    name: {
      hint: `a descriptive name for the account`,
      label: `account name账户名`
    },
    password: {
      hint: `a strong, unique password`,
      label: `password密码`
    },
    password2: {
      hint: `verify your password确认密码`,
      label: `password (repeat)再次输入密码`
    },
    phrase: {
      hint: `the account recovery phrase`,
      label: `account recovery phrase账户助记词`
    },
    windowsKey: {
      label: `Key was created with Parity <1.4.5 on Windows`
    }
  },
  title: {
    accountInfo: `account information账户信息`,
    createAccount: `create account创建账户`,
    createType: `creation type创建类型`,
    importWallet: `import wallet导入钱包`
  }
};
