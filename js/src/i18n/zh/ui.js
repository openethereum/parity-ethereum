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
  actionbar: {
    export: {
      button: {
        export: `导出`// export
      }
    },
    import: {
      button: {
        cancel: `取消`, // Cancel
        confirm: `确认`, // Confirm
        import: `导入`// import
      },
      confirm: `确认这是你想导入的`, // Confirm that this is what was intended to import.
      error: `发生错误：{errorText}`, // An error occured: {errorText}
      step: {
        error: `错误`, // error
        select: `选择一个文件`, // select a file
        validate: `确认`// validate
      },
      title: `从一个文件导入`// Import from a file
    },
    search: {
      hint: `输入搜索内容……`// Enter search input...
    },
    sort: {
      sortBy: `根据{label}排序`, // Sort by {label}
      typeDefault: `默认`, // Default
      typeEth: `根据以太币数额排序`, // Sort by ETH
      typeName: `根据账户名字排序`, // Sort by name
      typeTags: `根据标签排序`// Sort by tags
    }
  },
  balance: {
    none: `这个账户没有余额`// No balances associated with this account
  },
  blockStatus: {
    bestBlock: `最新区块{blockNumber}`, // {blockNumber} best block
    syncStatus: `currentBlock}/{highestBlock}区块同步`, // {currentBlock}/{highestBlock} syncing{
    warpRestore: `{percentage}%恢复`, // {percentage}% warp restore
    warpStatus: `, {percentage}%历史`// {percentage}% historic
  },
  confirmDialog: {
    no: `不是`, // no
    yes: `是`// yes
  },
  copyToClipboard: {
    copied: `复制{data}到粘贴板`// copied {data} to clipboard
  },
  errors: {
    close: `关闭`// close
  },
  fileSelect: {
    defaultLabel: `拉一个文件到这里，或者选择一个文件上传`// Drop a file here, or click to select a file to upload
  },
  gasPriceSelector: {
    customTooltip: {
      transactions: `{number} {number, plural, one {transaction} other {transactions}} with gas price set from {minPrice} to {maxPrice}`
    }
  },
  identityName: {
    null: `空`, // NULL
    unnamed: `未命名`// UNNAMED
  },
  methodDecoding: {
    condition: {
      block: `, {historic, select, true {Submitted} false {Submission}} at block {blockNumber}`,
      time: `, {historic, select, true {Submitted} false {Submission}} at {timestamp}`
    },
    deploy: {
      address: `在地址上部署一个合约`, // Deployed a contract at address
      params: `附带下面的参数：`, // with the following parameters:
      willDeploy: `将要部署一个合约`, // Will deploy a contract
      withValue: `, 发送{value}`// sending {value}
    },
    gasUsed: `({gas}gas消耗)`, // {gas} gas used
    gasValues: `{gas} gas ({gasPrice}M/{tag})`,
    input: {
      data: `数据`, // data
      input: `输入`, // input
      withInput: `with the {inputDesc} {inputValue}`
    },
    receive: {
      contract: `合约`, // the contract
      info: `{historic, select, true {Received} false {Will receive}} {valueEth} from {aContract}{address}`
    },
    signature: {
      info: `{historic, select, true {Executed} false {Will execute}} the {method} function on the contract {address} trsansferring {ethValue}{inputLength, plural, zero {,} other {passing the following {inputLength, plural, one {parameter} other {parameters}}}}`
    },
    token: {
      transfer: `{historic, select, true {Transferred} false {Will transfer}} {value} to {address}`
    },
    transfer: {
      contract: `the contract`,
      info: `{historic, select, true {Transferred} false {Will transfer}} {valueEth} to {aContract}{address}`
    },
    txValues: `{historic, select, true {Provided} false {Provides}} {gasProvided}{gasUsed} for a total transaction value of {totalEthValue}`,
    unknown: {
      info: `{historic, select, true {Executed} false {Will execute}} the {method} on the contract {address} transferring {ethValue}.`
    }
  },
  passwordStrength: {
    label: `密码强度`// password strength
  },
  tooltips: {
    button: {
      done: `完成`, // Done
      next: `下一步`, // Next
      skip: `跳过`// Skip
    }
  },
  txHash: {
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`,
    oog: `这笔交易已经耗光了gas。请用更多的gas尝试。`, // The transaction might have gone out of gas. Try again with more gas.
    posted: `这笔交易已经被发送到网络，附带哈希是{hashLink}`, // The transaction has been posted to the network with a hash of {hashLink}
    waiting: `等待确认`// waiting for confirmations
  },
  vaultSelect: {
    hint: `这个账户绑定的保险库是`, // the vault this account is attached to
    label: `相关保险库`// associated vault
  },
  verification: {
    gatherData: {
      accountHasRequested: {
        false: `.你还没有从这个账户请求确认。`, // You did not request verification from this account yet
        pending: `检查一下你是否请求了验证……`, // Checking if you requested verification…
        true: `你已经从这个账户请求到验证。`// You already requested verification from this account.
      },
      accountIsVerified: {
        false: `你的账户还没有被验证。`, // Your account is not verified yet.
        pending: `检查一下你的账户是否已经被验证……`, // Checking if your account is verified…
        true: `你的账户已经被验证。`// Your account is already verified.
      },
      fee: `额外的费用是{amount}ETH`, // The additional fee is {amount} ETH.
      isAbleToRequest: {
        pending: `验证你的输入……`// Validating your input…
      },
      isServerRunning: {
        false: `验证服务器没有在运行。`, // The verification server is not running.
        pending: `检查一下验证服务器是否在运行……`, // Checking if the verification server is running…
        true: `验证服务器正在运行。`// The verification server is running.
      },
      nofee: `没有额外的费用。`, // There is no additional fee.
      termsOfService: `我同意下面的条款和条件。`// I agree to the terms and conditions below.
    }
  }
};
