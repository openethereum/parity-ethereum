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
  button: {
    add: `Add`,
    cancel: `Cancel`,
    close: `Close`,
    create: `Create`,
    done: `Done`,
    next: `Next`,
    sending: `Sending...`
  },
  deployment: {
    message: `The deployment is currently in progress`
  },
  details: {
    address: {
      hint: `the wallet contract address`,
      label: `wallet address`
    },
    dayLimitMulti: {
      hint: `amount of ETH spendable without confirmations`,
      label: `wallet day limit`
    },
    description: {
      hint: `the local description for this wallet`,
      label: `wallet description (optional)`
    },
    descriptionMulti: {
      hint: `the local description for this wallet`,
      label: `wallet description (optional)`
    },
    name: {
      hint: `the local name for this wallet`,
      label: `wallet name`
    },
    nameMulti: {
      hint: `the local name for this wallet`,
      label: `wallet name`
    },
    ownerMulti: {
      hint: `the owner account for this contract`,
      label: `from account (contract owner)`
    },
    ownersMulti: {
      label: `other wallet owners`
    },
    ownersMultiReq: {
      hint: `number of required owners to accept a transaction`,
      label: `required owners`
    }
  },
  info: {
    added: `added`,
    copyAddress: `copy address to clipboard`,
    created: `{name} has been {deployedOrAdded} at`,
    dayLimit: `The daily limit is set to {dayLimit} ETH.`,
    deployed: `deployed`,
    numOwners: `{numOwners} owners are required to confirm a transaction.`,
    owners: `The following are wallet owners`
  },
  rejected: {
    message: `The deployment has been rejected`,
    state: `The wallet will not be created. You can safely close this window.`,
    title: `rejected`
  },
  states: {
    completed: `The contract deployment has been completed`,
    confirmationNeeded: `The contract deployment needs confirmations from other owners of the Wallet`,
    preparing: `Preparing transaction for network transmission`,
    validatingCode: `Validating the deployed contract code`,
    waitingConfirm: `Waiting for confirmation of the transaction in the Parity Secure Signer`,
    waitingReceipt: `Waiting for the contract deployment transaction receipt`
  },
  steps: {
    deployment: `wallet deployment`,
    details: `wallet details`,
    info: `wallet informaton`,
    type: `wallet type`
  },
  type: {
    multisig: {
      description: `Create/Deploy a {link} Wallet`,
      label: `Multi-Sig wallet`,
      link: `standard multi-signature`
    },
    watch: {
      description: `Add an existing wallet to your accounts`,
      label: `Watch a wallet`
    }
  }
};
