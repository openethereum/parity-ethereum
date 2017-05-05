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
        export: `匯出`// export
      }
    },
    import: {
      button: {
        cancel: `取消`, // Cancel
        confirm: `確認`, // Confirm
        import: `匯入`// import
      },
      confirm: `確認這是你想匯入的`, // Confirm that this is what was intended to import.
      error: `發生錯誤：{errorText}`, // An error occured: {errorText}
      step: {
        error: `錯誤`, // error
        select: `選擇一個檔案`, // select a file
        validate: `確認`// validate
      },
      title: `從一個檔案匯入`// Import from a file
    },
    search: {
      hint: `輸入搜尋內容……`// Enter search input...
    },
    sort: {
      sortBy: `根據{label}排序`, // Sort by {label}
      typeDefault: `預設`, // Default
      typeEth: `根據以太幣數額排序`, // Sort by ETH
      typeName: `根據帳戶名字排序`, // Sort by name
      typeTags: `根據標籤排序`// Sort by tags
    }
  },
  balance: {
    none: `這個帳戶沒有餘額`// No balances associated with this account
  },
  blockStatus: {
    bestBlock: `最新區塊{blockNumber}`, // {blockNumber} best block
    syncStatus: `currentBlock}/{highestBlock}區塊同步`, // {currentBlock}/{highestBlock} syncing{
    warpRestore: `{percentage}%恢復`, // {percentage}% warp restore
    warpStatus: `, {percentage}%歷史`// {percentage}% historic
  },
  confirmDialog: {
    no: `不是`, // no
    yes: `是`// yes
  },
  copyToClipboard: {
    copied: `複製{data}到貼上板`// copied {data} to clipboard
  },
  errors: {
    close: `關閉`// close
  },
  fileSelect: {
    defaultLabel: `拉一個檔案到這裡，或者選擇一個檔案上傳`// Drop a file here, or click to select a file to upload
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
      address: `在地址上部署一個合約`, // Deployed a contract at address
      params: `附帶下面的引數：`, // with the following parameters:
      willDeploy: `將要部署一個合約`, // Will deploy a contract
      withValue: `, 傳送{value}`// sending {value}
    },
    gasUsed: `({gas}gas消耗)`, // {gas} gas used
    gasValues: `{gas} gas ({gasPrice}M/{tag})`,
    input: {
      data: `資料`, // data
      input: `輸入`, // input
      withInput: `with the {inputDesc} {inputValue}`
    },
    receive: {
      contract: `合約`, // the contract
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
    label: `密碼強度`// password strength
  },
  tooltips: {
    button: {
      done: `完成`, // Done
      next: `下一步`, // Next
      skip: `跳過`// Skip
    }
  },
  txHash: {
    confirmations: `{count} {value, plural, one {confirmation} other {confirmations}}`,
    oog: `這筆交易已經耗光了gas。請用更多的gas嘗試。`, // The transaction might have gone out of gas. Try again with more gas.
    posted: `這筆交易已經被髮送到網路，附帶雜湊是{hashLink}`, // The transaction has been posted to the network with a hash of {hashLink}
    waiting: `等待確認`// waiting for confirmations
  },
  vaultSelect: {
    hint: `這個帳戶繫結的保險庫是`, // the vault this account is attached to
    label: `相關保險庫`// associated vault
  },
  verification: {
    gatherData: {
      accountHasRequested: {
        false: `.你還沒有從這個帳戶請求確認。`, // You did not request verification from this account yet
        pending: `檢查一下你是否請求了驗證……`, // Checking if you requested verification…
        true: `你已經從這個帳戶請求到驗證。`// You already requested verification from this account.
      },
      accountIsVerified: {
        false: `你的帳戶還沒有被驗證。`, // Your account is not verified yet.
        pending: `檢查一下你的帳戶是否已經被驗證……`, // Checking if your account is verified…
        true: `你的帳戶已經被驗證。`// Your account is already verified.
      },
      fee: `額外的費用是{amount}ETH`, // The additional fee is {amount} ETH.
      isAbleToRequest: {
        pending: `驗證你的輸入……`// Validating your input…
      },
      isServerRunning: {
        false: `驗證伺服器沒有在執行。`, // The verification server is not running.
        pending: `檢查一下驗證伺服器是否在執行……`, // Checking if the verification server is running…
        true: `驗證伺服器正在執行。`// The verification server is running.
      },
      nofee: `沒有額外的費用。`, // There is no additional fee.
      termsOfService: `我同意下面的條款和條件。`// I agree to the terms and conditions below.
    }
  }
};
