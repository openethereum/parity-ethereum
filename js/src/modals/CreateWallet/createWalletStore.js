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

import { observable, computed, action, transaction } from 'mobx';
import React from 'react';
import { FormattedMessage } from 'react-intl';

import Contract from '~/api/contract';
import { ERROR_CODES } from '~/api/transport/error';
import Contracts from '~/contracts';
import { wallet as walletAbi } from '~/contracts/abi';
import { wallet as walletCode, walletLibraryRegKey, fullWalletCode } from '~/contracts/code/wallet';

import { validateUint, validateAddress, validateName } from '~/util/validation';
import { toWei } from '~/api/util/wei';
import WalletsUtils from '~/util/wallets';

const STEPS = {
  TYPE: {
    title: (
      <FormattedMessage
        id='createWallet.steps.type'
        defaultMessage='wallet type'
      />
    )
  },
  DETAILS: {
    title: (
      <FormattedMessage
        id='createWallet.steps.details'
        defaultMessage='wallet details'
      />
    )
  },
  DEPLOYMENT: {
    title: (
      <FormattedMessage
        id='createWallet.steps.deployment'
        defaultMessage='wallet deployment'
      />
    ),
    waiting: true
  },
  INFO: {
    title: (
      <FormattedMessage
        id='createWallet.steps.info'
        defaultMessage='wallet informaton'
      />
    )
  }
};

export default class CreateWalletStore {
  @observable step = null;
  @observable rejected = false;

  @observable deployState = null;
  @observable deployError = null;
  @observable deployed = false;

  @observable txhash = null;

  @observable wallet = {
    account: '',
    address: '',
    owners: [],
    required: 1,
    daylimit: toWei(1),

    name: '',
    description: ''
  };
  @observable walletType = 'MULTISIG';

  @observable errors = {
    account: null,
    address: null,
    owners: null,
    required: null,
    daylimit: null,
    name: null
  };

  @computed get stage () {
    return this.stepsKeys.findIndex((k) => k === this.step);
  }

  @computed get hasErrors () {
    return !!Object.keys(this.errors)
      .filter((errorKey) => {
        if (this.walletType === 'WATCH') {
          return ['address', 'name'].includes(errorKey);
        }

        return errorKey !== 'address';
      })
      .find((key) => !!this.errors[key]);
  }

  @computed get stepsKeys () {
    return this.steps.map((s) => s.key);
  }

  @computed get steps () {
    return Object
      .keys(STEPS)
      .map((key) => {
        return {
          ...STEPS[key],
          key
        };
      })
      .filter((step) => {
        return (this.walletType !== 'WATCH' || step.key !== 'DEPLOYMENT');
      });
  }

  @computed get waiting () {
    this.steps
      .map((s, idx) => ({ idx, waiting: s.waiting }))
      .filter((s) => s.waiting)
      .map((s) => s.idx);
  }

  constructor (api, accounts) {
    this.api = api;

    this.step = this.stepsKeys[0];
    this.wallet.account = Object.values(accounts)[0].address;
    this.validateWallet(this.wallet);
  }

  @action onTypeChange = (type) => {
    this.walletType = type;
    this.validateWallet(this.wallet);
  }

  @action onNext = () => {
    const stepIndex = this.stepsKeys.findIndex((k) => k === this.step) + 1;

    this.step = this.stepsKeys[stepIndex];
  }

  @action onChange = (_wallet) => {
    const newWallet = Object.assign({}, this.wallet, _wallet);

    this.validateWallet(newWallet);
  }

  @action onAdd = () => {
    if (this.hasErrors) {
      return;
    }

    const walletContract = new Contract(this.api, walletAbi).at(this.wallet.address);

    return Promise
      .all([
        WalletsUtils.fetchRequire(walletContract),
        WalletsUtils.fetchOwners(walletContract),
        WalletsUtils.fetchDailylimit(walletContract)
      ])
      .then(([ require, owners, dailylimit ]) => {
        transaction(() => {
          this.wallet.owners = owners;
          this.wallet.required = require.toNumber();
          this.wallet.dailylimit = dailylimit.limit;
        });

        return this.addWallet(this.wallet);
      });
  }

  @action onCreate = () => {
    if (this.hasErrors) {
      return;
    }

    this.step = 'DEPLOYMENT';

    const { account, owners, required, daylimit } = this.wallet;

    Contracts
      .get()
      .registry
      .lookupAddress(walletLibraryRegKey)
      .catch(() => {
        return null; // exception when registry is not available
      })
      .then((address) => {
        const walletLibraryAddress = (address || '').replace(/^0x/, '').toLowerCase();
        const code = walletLibraryAddress.length && !/^0+$/.test(walletLibraryAddress)
          ? walletCode.replace(/(_)+WalletLibrary(_)+/g, walletLibraryAddress)
          : fullWalletCode;

        const options = {
          data: code,
          from: account
        };

        return this.api
          .newContract(walletAbi)
          .deploy(options, [ owners, required, daylimit ], this.onDeploymentState);
      })
      .then((address) => {
        this.deployed = true;
        this.wallet.address = address;
        return this.addWallet(this.wallet);
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

  @action addWallet = (wallet) => {
    const { address, name, description } = wallet;

    return Promise
      .all([
        this.api.parity.setAccountName(address, name),
        this.api.parity.setAccountMeta(address, {
          abi: walletAbi,
          wallet: true,
          timestamp: Date.now(),
          deleted: false,
          description,
          name,
          tags: ['wallet']
        })
      ])
      .then(() => {
        this.step = 'INFO';
      });
  }

  onDeploymentState = (error, data) => {
    if (error) {
      return console.error('createWallet::onDeploymentState', error);
    }

    switch (data.state) {
      case 'estimateGas':
      case 'postTransaction':
        this.deployState = (
          <FormattedMessage
            id='createWallet.states.preparing'
            defaultMessage='Preparing transaction for network transmission'
          />
        );
        return;

      case 'checkRequest':
        this.deployState = (
          <FormattedMessage
            id='createWallet.states.waitingConfirm'
            defaultMessage='Waiting for confirmation of the transaction in the Parity Secure Signer'
          />
        );
        return;

      case 'getTransactionReceipt':
        this.deployState = (
          <FormattedMessage
            id='createWallet.states.waitingReceipt'
            defaultMessage='Waiting for the contract deployment transaction receipt'
          />
        );
        this.txhash = data.txhash;
        return;

      case 'hasReceipt':
      case 'getCode':
        this.deployState = (
          <FormattedMessage
            id='createWallet.states.validatingCode'
            defaultMessage='Validating the deployed contract code'
          />
        );
        return;

      case 'completed':
        this.deployState = (
          <FormattedMessage
            id='createWallet.states.completed'
            defaultMessage='The contract deployment has been completed'
          />
        );
        return;

      default:
        console.error('createWallet::onDeploymentState', 'unknow contract deployment state', data);
        return;
    }
  }

  @action validateWallet = (_wallet) => {
    const addressValidation = validateAddress(_wallet.address);
    const accountValidation = validateAddress(_wallet.account);
    const requiredValidation = validateUint(_wallet.required);
    const daylimitValidation = validateUint(_wallet.daylimit);
    const nameValidation = validateName(_wallet.name);

    const errors = {
      address: addressValidation.addressError,
      account: accountValidation.addressError,
      required: requiredValidation.valueError,
      daylimit: daylimitValidation.valueError,
      name: nameValidation.nameError
    };

    const wallet = {
      ..._wallet,
      address: addressValidation.address,
      account: accountValidation.address,
      required: requiredValidation.value,
      daylimit: daylimitValidation.value,
      name: nameValidation.name
    };

    transaction(() => {
      this.wallet = wallet;
      this.errors = errors;
    });
  }
}
