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
      hint: `账户地址`,
      label: `地址`
    },
    name: {
      hint: `描述账户的名字`,
      label: `账户名`
    },
    phrase: {
      hint: `账户恢复词`,
      label: `账户恢复词（安全保存，别人拥有它就可以完全控制你的账户）`
    }
  },
  accountDetailsGeth: {
    imported: `你已经从Geth keystore导入了{number}个地址`
  },
  button: {
    back: `返回`,
    cancel: `取消`,
    close: `关闭`,
    create: `创建`,
    import: `导入`,
    next: `下一步`,
    print: `打印恢复词`
  },
  creationType: {
    fromGeth: {
      label: `从Geth keystore导入账户`
    },
    fromJSON: {
      label: `从JSON文件导入账户`
    },
    fromNew: {
      label: `手动创建新账户`
    },
    fromPhrase: {
      label: `通过恢复词恢复账户`
    },
    fromPresale: {
      label: `从以太坊预售钱包导入账户`
    },
    fromRaw: {
      label: `导入私钥`
    }
  },
  newAccount: {
    hint: {
      hint: `（可选）帮助你记住密码的提示`,
      label: `密码提示`
    },
    name: {
      hint: `描述账户的名字`,
      label: `账户名`
    },
    password: {
      hint: `足够强的密码`,
      label: `密码`
    },
    password2: {
      hint: `确认你的密码`,
      label: `再次输入密码`
    }
  },
  newGeth: {
    noKeys: `现在Geth keystore中没有可导入的私钥`
  },
  newImport: {
    file: {
      hint: `要导入的钱包文件`,
      label: `钱包文件`
    },
    hint: {
      hint: `（可选）帮助你记住密码的提示`,
      label: `密码提示`
    },
    name: {
      hint: `描述账户的名字`,
      label: `账户名`
    },
    password: {
      hint: `输入密码，解锁钱包`,
      label: `密码`
    }
  },
  rawKey: {
    hint: {
      hint: `（可选）帮助你记住密码的提示`,
      label: `密码提示`
    },
    name: {
      hint: `描述账户的名字`,
      label: `账户名`
    },
    password: {
      hint: `足够强的密码`,
      label: `密码`
    },
    password2: {
      hint: `确认密码`,
      label: `再次输入密码`
    },
    private: {
      hint: `原始的十六进制编码私钥`,
      label: `私钥`
    }
  },
  recoveryPhrase: {
    hint: {
      hint: `（可选）帮助你记住密码的提示`,
      label: `密码提示`
    },
    name: {
      hint: `描述账户的名字`,
      label: `账户名`
    },
    password: {
      hint: `足够强的密码`,
      label: `密码`
    },
    password2: {
      hint: `确认密码`,
      label: `再次输入密码`
    },
    phrase: {
      hint: `账户恢复词`,
      label: `账户恢复词`
    },
    windowsKey: {
      label: `在Windows系统上由Parity 1.4.5以前的版本创建的私钥`
    }
  },
  title: {
    accountInfo: `账户信息`,
    createAccount: `创建账户`,
    createType: `创建类型`,
    importWallet: `导入钱包`
  }
};
