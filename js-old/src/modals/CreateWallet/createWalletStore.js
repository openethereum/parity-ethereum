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

import { noop } from 'lodash';
import { observable, computed, action, transaction } from 'mobx';
import React from 'react';
import { FormattedMessage } from 'react-intl';

import Contract from '~/api/contract';
import Contracts from '~/contracts';
import { foundationWallet as walletAbi } from '~/contracts/abi';
import { wallet as walletCode, walletLibrary as walletLibraryCode, walletLibraryRegKey, fullWalletCode } from '~/contracts/code/wallet';

import { validateUint, validateAddress, validateName } from '~/util/validation';
import { toWei } from '~/api/util/wei';
import { deploy, getSender, loadSender, setSender } from '~/util/tx';
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
  @observable txhash = null;
  @observable walletType = 'MULTISIG';

  @observable wallet = {
    account: '',
    address: '',
    owners: [],
    required: 1,
    daylimit: toWei(1),

    name: '',
    description: ''
  };

  @observable errors = {
    account: null,
    address: null,
    owners: null,
    required: null,
    daylimit: null,
    name: null
  };

  onClose = noop;
  onSetRequest = noop;

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
      .filter((step) => this.walletType === 'WATCH' || step.key !== 'INFO');
  }

  constructor (api, { accounts, onClose, onSetRequest }) {
    this.api = api;

    this.step = this.stepsKeys[0];
    this.wallet.account = getSender() || Object.values(accounts)[0].address;
    this.validateWallet(this.wallet);
    this.onClose = onClose;
    this.onSetRequest = onSetRequest;

    loadSender(this.api)
      .then((defaultAccount) => {
        if (defaultAccount !== this.wallet.account) {
          this.onChange({ account: defaultAccount });
        }
      });
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
      .then(([ require, owners, daylimit ]) => {
        transaction(() => {
          this.wallet.owners = owners;
          this.wallet.required = require.toNumber();
          this.wallet.daylimit = daylimit.limit;

          this.wallet = this.getWalletWithMeta(this.wallet);
        });

        return this.addWallet(this.wallet);
      });
  }

  @action onCreate = () => {
    if (this.hasErrors) {
      return;
    }

    const { account, owners, required, daylimit } = this.wallet;

    Contracts
      .get()
      .registry
      .lookupAddress(walletLibraryRegKey)
      .catch(() => {
        return null; // exception when registry is not available
      })
      .then((address) => {
        console.warn('WalletLibrary address in registry', address);

        if (!address || /^(0x)?0*$/.test(address)) {
          return null;
        }

        // Check that it's actually the expected code
        return this.api.eth
          .getCode(address)
          .then((code) => {
            const strippedCode = code.replace(/^0x/, '');

            // The actual deployed code is included in the wallet
            // library code (which might have some more data)
            if (walletLibraryCode.indexOf(strippedCode) >= 0) {
              return address;
            }

            return null;
          });
      })
      .then((address) => {
        let code = fullWalletCode;

        if (address) {
          const walletLibraryAddress = address.replace(/^0x/, '').toLowerCase();

          code = walletCode.replace(/(_)+WalletLibrary(_)+/g, walletLibraryAddress);
        } else {
          console.warn('wallet library has not been found in the registry');
        }

        const options = {
          data: code,
          from: account
        };

        const contract = this.api.newContract(walletAbi);

        setSender(account);
        this.wallet = this.getWalletWithMeta(this.wallet);
        this.onClose();
        return deploy(contract, options, [ owners, required, daylimit ])
          .then((requestId) => {
            const metadata = { ...this.wallet.metadata, deployment: true };

            this.onSetRequest(requestId, { metadata }, false);
          });
      });
  }

  @action addWallet = (wallet) => {
    const { address, name, metadata } = wallet;

    return Promise
      .all([
        this.api.parity.setAccountName(address, name),
        this.api.parity.setAccountMeta(address, metadata)
      ])
      .then(() => {
        this.step = 'INFO';
      });
  }

  getWalletWithMeta = (wallet) => {
    const { name, description } = wallet;

    const metadata = {
      abi: walletAbi,
      wallet: true,
      timestamp: Date.now(),
      deleted: false,
      tags: [ 'wallet' ],
      description,
      name
    };

    return {
      ...wallet,
      metadata
    };
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
