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
  rejected: {
    title: `rejected`,
    message: `The deployment has been rejected`,
    state: `The wallet will not be created. You can safely close this window.`
  },
  deployment: {
    message: `The deployment is currently in progress`
  },
  button: {
    cancel: `Cancel`,
    close: `Close`,
    done: `Done`,
    sending: `Sending...`,
    next: `Next`,
    add: `Add`,
    create: `Create`
  },
  steps: {
    type: `wallet type`,
    details: `wallet details`,
    deployment: `wallet deployment`,
    info: `wallet informaton`
  },
  states: {
    preparing: `Preparing transaction for network transmission`,
    waitingConfirm: `Waiting for confirmation of the transaction in the Parity Secure Signer`,
    waitingReceipt: `Waiting for the contract deployment transaction receipt`,
    validatingCode: `Validating the deployed contract code`,
    completed: `The contract deployment has been completed`
  },
  details: {
    address: {
      hint: `the wallet contract address`,
      label: `wallet address`
    },
    name: {
      hint: `the local name for this wallet`,
      label: `wallet name`
    },
    description: {
      hint: `the local description for this wallet`,
      label: `wallet description (optional)`
    },
    ownerMulti: {
      hint: `the owner account for this contract`,
      label: `from account (contract owner)`
    },
    nameMulti: {
      hint: `the local name for this wallet`,
      label: `wallet name`
    },
    descriptionMulti: {
      hint: `the local description for this wallet`,
      label: `wallet description (optional)`
    },
    ownersMulti: {
      label: `other wallet owners`
    },
    ownersMultiReq: {
      hint: `number of required owners to accept a transaction`,
      label: `required owners`
    },
    dayLimitMulti: {
      hint: `amount of ETH spendable without confirmations`,
      label: `wallet day limit`
    }
  },
  info: {
    created: `{name} has been {deployedOrAdded} at`,
    deployed: `deployed`,
    added: `added`,
    copyAddress: `copy address to clipboard`,
    owners: `The following are wallet owners`,
    numOwners: `{numOwners} owners are required to confirm a transaction.`,
    dayLimit: `The daily limit is set to {dayLimit} ETH.`
  },
  type: {
    multisig: {
      label: `Multi-Sig wallet`,
      description: `Create/Deploy a {link} Wallet`,
      link: `standard multi-signature`
    },
    watch: {
      label: `Watch a wallet`,
      description: `Add an existing wallet to your accounts`
    }
  }
};
