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
      hint: `帳戶地址`,
      label: `地址`
    },
    name: {
      hint: `描述帳戶的名字`,
      label: `帳戶名`
    },
    phrase: {
      hint: `帳戶恢復詞`,
      label: `帳戶恢復詞（安全儲存，別人擁有它就可以完全控制你的帳戶）`
    }
  },
  accountDetailsGeth: {
    imported: `你已經從Geth keystore匯入了{number}個地址`
  },
  button: {
    back: `返回`,
    cancel: `取消`,
    close: `關閉`,
    create: `建立`,
    done: `Done`,
    import: `匯入`,
    next: `下一步`,
    print: `列印恢復詞`
  },
  creationType: {
    fromGeth: {
      description: `Import accounts from the Geth keystore with the original password`,
      label: `從Geth keystore匯入帳戶`
    },
    fromJSON: {
      description: `Import an industry-standard JSON keyfile with the original password`,
      label: `從JSON檔案匯入帳戶`
    },
    fromNew: {
      description: `Selecting your identity icon and specifying the password`,
      label: `手動建立新帳戶`
    },
    fromPhrase: {
      description: `Recover using a previously stored recovery phrase and new password`,
      label: `通過恢復詞恢復帳戶`
    },
    fromPresale: {
      description: `Import an Ethereum presale wallet file with the original password`,
      label: `從以太坊預售錢包匯入帳戶`
    },
    fromRaw: {
      description: `Enter a previously created raw private key with a new password`,
      label: `匯入私鑰`
    },
    info: `Please select the type of account you want to create. Either create an account via name & password, or import it from a variety of existing sources. From here the wizard will guide you through the process of completing your account creation.`
  },
  newAccount: {
    hint: {
      hint: `（可選）幫助你記住密碼的提示`,
      label: `密碼提示`
    },
    name: {
      hint: `描述帳戶的名字`,
      label: `帳戶名`
    },
    password: {
      hint: `足夠強的密碼`,
      label: `密碼`
    },
    password2: {
      hint: `確認你的密碼`,
      label: `再次輸入密碼`
    }
  },
  newGeth: {
    available: `There are currently {count} importable keys available from the Geth keystore which are not already available on your Parity instance. Select the accounts you wish to import and move to the next step to complete the import.`,
    noKeys: `現在Geth keystore中沒有可匯入的私鑰`
  },
  newImport: {
    file: {
      hint: `要匯入的錢包檔案`,
      label: `錢包檔案`
    },
    hint: {
      hint: `（可選）幫助你記住密碼的提示`,
      label: `密碼提示`
    },
    name: {
      hint: `描述帳戶的名字`,
      label: `帳戶名`
    },
    password: {
      hint: `輸入密碼，解鎖錢包`,
      label: `密碼`
    }
  },
  rawKey: {
    hint: {
      hint: `（可選）幫助你記住密碼的提示`,
      label: `密碼提示`
    },
    name: {
      hint: `描述帳戶的名字`,
      label: `帳戶名`
    },
    password: {
      hint: `足夠強的密碼`,
      label: `密碼`
    },
    password2: {
      hint: `確認密碼`,
      label: `再次輸入密碼`
    },
    private: {
      hint: `原始的十六進位制編碼私鑰`,
      label: `私鑰`
    }
  },
  recoveryPhrase: {
    hint: {
      hint: `（可選）幫助你記住密碼的提示`,
      label: `密碼提示`
    },
    name: {
      hint: `描述帳戶的名字`,
      label: `帳戶名`
    },
    password: {
      hint: `足夠強的密碼`,
      label: `密碼`
    },
    password2: {
      hint: `確認密碼`,
      label: `再次輸入密碼`
    },
    phrase: {
      hint: `帳戶恢復詞`,
      label: `帳戶恢復詞`
    },
    windowsKey: {
      label: `在Windows系統上由Parity 1.4.5以前的版本建立的私鑰`
    }
  },
  title: {
    accountInfo: `帳戶資訊`,
    createAccount: `建立帳戶`,
    createType: `建立型別`,
    importWallet: `匯入錢包`
  }
};
