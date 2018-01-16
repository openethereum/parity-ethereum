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
        export: `export`
      }
    },
    import: {
      button: {
        cancel: `Cancel`,
        confirm: `Confirm`,
        import: `import`
      },
      confirm: `Confirm that this is what was intended to import.`,
      error: `An error occured: {errorText}`,
      step: {
        error: `error`,
        select: `select a file`,
        validate: `validate`
      },
      title: `Import from a file`
    },
    search: {
      hint: `Enter search input...`
    },
    sort: {
      sortBy: `Sort by {label}`,
      typeDefault: `Default`,
      typeEth: `Sort by ETH`,
      typeName: `Sort by name`,
      typeTags: `Sort by tags`
    }
  },
  balance: {
    none: `No balances associated with this account`
  },
  blockStatus: {
    bestBlock: `{blockNumber} best block`,
    syncStatus: `{currentBlock}/{highestBlock} syncing`,
    warpRestore: `{percentage}% warp restore`,
    warpStatus: `, {percentage}% historic`
  },
  confirmDialog: {
    no: `no`,
    yes: `yes`
  },
  copyToClipboard: {
    copied: `copied {data} to clipboard`
  },
  errors: {
    close: `close`
  },
  fileSelect: {
    defaultLabel: `Drop a file here, or click to select a file to upload`
  },
  gasPriceSelector: {
    customTooltip: {
      transactions: `{number} {number, plural, one {transaction} other {transactions}} with gas price set from {minPrice} to {maxPrice}`
    }
  },
  identityName: {
    null: `NULL`,
    unnamed: `UNNAMED`
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
      data: `data`,
      input: `input`,
      withInput: `with the {inputDesc} {inputValue}`
    },
    receive: {
      contract: `the contract`,
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
    label: `password strength`
  },
  tooltips: {
    button: {
      done: `Done`,
      next: `Next`,
      skip: `Skip`
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
        false: `You did not request verification from this account yet.`,
        pending: `Checking if you requested verification…`,
        true: `You already requested verification from this account.`
      },
      accountIsVerified: {
        false: `Your account is not verified yet.`,
        pending: `Checking if your account is verified…`,
        true: `Your account is already verified.`
      },
      fee: `The additional fee is {amount} ETH.`,
      isAbleToRequest: {
        pending: `Validating your input…`
      },
      isServerRunning: {
        false: `The verification server is not running.`,
        pending: `Checking if the verification server is running…`,
        true: `The verification server is running.`
      },
      nofee: `There is no additional fee.`,
      termsOfService: `I agree to the terms and conditions below.`
    }
  }
};
