// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { observable, computed, action, transaction } from 'mobx';

import { ERRORS, validateUint, validateAddress, validateName } from '../../util/validation';
import { ERROR_CODES } from '../../api/transport/error';

import { wallet as walletAbi } from '../../contracts/abi';
import { wallet as walletCode } from '../../contracts/code';

const STEPS = {
  DETAILS: { title: 'wallet details' },
  DEPLOYMENT: { title: 'wallet deployment', waiting: true },
  INFO: { title: 'wallet informaton' }
};

const STEPS_KEYS = Object.keys(STEPS);

export default class CreateWalletStore {
  @observable step = null;
  @observable rejected = false;

  @observable deployState = null;
  @observable deployError = null;

  @observable txhash = null;

  @observable wallet = {
    account: '',
    address: '',
    owners: [],
    required: 1,
    daylimit: 0,

    name: '',
    description: ''
  };

  @observable errors = {
    account: null,
    owners: null,
    required: null,
    daylimit: null,

    name: ERRORS.invalidName
  };

  @computed get stage () {
    return STEPS_KEYS.findIndex((k) => k === this.step);
  }

  @computed get hasErrors () {
    return !!Object.values(this.errors).find((e) => !!e);
  }

  steps = Object.values(STEPS).map((s) => s.title);
  waiting = Object.values(STEPS)
    .map((s, idx) => ({ idx, waiting: s.waiting }))
    .filter((s) => s.waiting)
    .map((s) => s.idx);

  constructor (api, accounts) {
    this.api = api;

    this.step = STEPS_KEYS[0];
    this.wallet.account = Object.values(accounts)[0].address;
  }

  @action onChange = (_wallet) => {
    const newWallet = Object.assign({}, this.wallet, _wallet);
    const { errors, wallet } = this.validateWallet(newWallet);

    transaction(() => {
      this.wallet = wallet;
      this.errors = errors;
    });
  }

  @action onCreate = () => {
    if (this.hasErrors) {
      return;
    }

    this.step = 'DEPLOYMENT';

    const { account, owners, required, daylimit, name, description } = this.wallet;

    const options = {
      data: walletCode,
      from: account
    };

    this.api
      .newContract(walletAbi)
      .deploy(options, [ owners, required, daylimit ], this.onDeploymentState)
      .then((address) => {
        return Promise
          .all([
            this.api.parity.setAccountName(address, name),
            this.api.parity.setAccountMeta(address, {
              abi: walletAbi,
              wallet: true,
              timestamp: Date.now(),
              deleted: false,
              description,
              name
            })
          ])
          .then(() => {
            transaction(() => {
              this.wallet.address = address;
              this.step = 'INFO';
            });
          });
      })
      .catch((error) => {
        if (error.code === ERROR_CODES.REQUEST_REJECTED) {
          this.rejected = true;
          return;
        }

        console.error('error deploying contract', error);
        this.deployError = error;
      });
  }

  onDeploymentState = (error, data) => {
    if (error) {
      return console.error('createWallet::onDeploymentState', error);
    }

    switch (data.state) {
      case 'estimateGas':
      case 'postTransaction':
        this.deployState = 'Preparing transaction for network transmission';
        return;

      case 'checkRequest':
        this.deployState = 'Waiting for confirmation of the transaction in the Parity Secure Signer';
        return;

      case 'getTransactionReceipt':
        this.deployState = 'Waiting for the contract deployment transaction receipt';
        this.txhash = data.txhash;
        return;

      case 'hasReceipt':
      case 'getCode':
        this.deployState = 'Validating the deployed contract code';
        return;

      case 'completed':
        this.deployState = 'The contract deployment has been completed';
        return;

      default:
        console.error('createWallet::onDeploymentState', 'unknow contract deployment state', data);
        return;
    }
  }

  validateWallet = (_wallet) => {
    const accountValidation = validateAddress(_wallet.account);
    const requiredValidation = validateUint(_wallet.required);
    const daylimitValidation = validateUint(_wallet.daylimit);
    const nameValidation = validateName(_wallet.name);

    const errors = {
      account: accountValidation.addressError,
      required: requiredValidation.valueError,
      daylimit: daylimitValidation.valueError,
      name: nameValidation.nameError
    };

    const wallet = {
      ..._wallet,
      account: accountValidation.address,
      required: requiredValidation.value,
      daylimit: daylimitValidation.value,
      name: nameValidation.name
    };

    return { errors, wallet };
  }
}
