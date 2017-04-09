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
        export: `export导出`
      }
    },
    import: {
      button: {
        cancel: `Cancel取消`,
        confirm: `Confirm确认`,
        import: `import导入`
      },
      confirm: `Confirm that this is what was intended to import.确认这是你想导入的`,
      error: `An error occured: {errorText}发生错误：{errorText}`,
      step: {
        error: `error错误`,
        select: `select a file选择一个文件`,
        validate: `validate确认`
      },
      title: `Import from a file从一个文件导入`
    },
    search: {
      hint: `Enter search input...输入搜索内容……`
    },
    sort: {
      sortBy: `Sort by {label}根据{label}排序`,
      typeDefault: `Default默认`,
      typeEth: `Sort by ETH根据以太币数额排序`,
      typeName: `Sort by name根据账户名字排序`,
      typeTags: `Sort by tags根据标签排序`
    }
  },
  balance: {
    none: `No balances associated with this account这个账户没有余额`
  },
  blockStatus: {
    bestBlock: `{blockNumber} best block最新区块{blockNumber}`,
    syncStatus: `{currentBlock}/{highestBlock} syncing`,
    warpRestore: `{percentage}% warp restore`,
    warpStatus: `, {percentage}% historic`
  },
  confirmDialog: {
    no: `no不是`,
    yes: `yes是`
  },
  copyToClipboard: {
    copied: `copied {data} to clipboard复制{data}到粘贴板`
  },
  errors: {
    close: `close关闭`
  },
  fileSelect: {
    defaultLabel: `Drop a file here, or click to select a file to upload拉一个文件到这里，或者选择一个文件上传`
  },
  gasPriceSelector: {
    customTooltip: {
      transactions: `{number} {number, plural, one {transaction} other {transactions}} with gas price set from {minPrice} to {maxPrice}`
    }
  },
  identityName: {
    null: `NULL空`,
    unnamed: `UNNAMED未命名`
  },
  methodDecoding: {
    condition: {
      block: `, {historic, select, true {Submitted} false {Submission}} at block {blockNumber}`,
      time: `, {historic, select, true {Submitted} false {Submission}} at {timestamp}`
    },
    deploy: {
      address: `Deployed a contract at address`,
      params: `with the following parameters:`,
      willDeploy: `Will deploy a contract`,
      withValue: `, sending {value}`
    },
    gasUsed: `({gas} gas used)`,
    gasValues: `{gas} gas ({gasPrice}M/{tag})`,
    input: {
      data: `data数据`,
      input: `input输入`,
      withInput: `with the {inputDesc} {inputValue}`
    },
    receive: {
      contract: `the contract合约`,
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
    label: `password strength密码强度`
  },
  tooltips: {
    button: {
      done: `Done完成`,
      next: `Next下一步`,
      skip: `Skip跳过`
    }
  },
  txHash: {
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`,
    oog: `The transaction might have gone out of gas. Try again with more gas.`,
    posted: `The transaction has been posted to the network with a hash of {hashLink}`,
    waiting: `waiting for confirmations`
  },
  vaultSelect: {
    hint: `the vault this account is attached to`,
    label: `associated vault`
  },
  verification: {
    gatherData: {
      accountHasRequested: {
        false: `You did not request verification from this account yet.你还没有从这个账户请求确认。`,
        pending: `Checking if you requested verification…检查一下你是否请求了验证……`,
        true: `You already requested verification from this account.`
      },
      accountIsVerified: {
        false: `Your account is not verified yet.你的账户还没有被验证。`,
        pending: `Checking if your account is verified…检查一下你的账户是否已经被验证……`,
        true: `Your account is already verified.你的账户已经被验证。`
      },
      fee: `The additional fee is {amount} ETH.额外的费用是{amount}ETH`,
      isAbleToRequest: {
        pending: `Validating your input…验证你的输入……`
      },
      isServerRunning: {
        false: `The verification server is not running.验证服务器没有在运行。`,
        pending: `Checking if the verification server is running…检查一下验证服务器是否在运行……`,
        true: `The verification server is running.验证服务器正在运行。`
      },
      nofee: `There is no additional fee.没有额外的费用。`,
      termsOfService: `I agree to the terms and conditions below.我同意下面的条款和条件。`
    }
  }
};
