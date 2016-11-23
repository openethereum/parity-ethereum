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

import { observable, computed, autorun, action } from 'mobx';
import phone from 'phoneformat.js';
import { sha3 } from '../../api/util/sha3';

import Contracts from '../../contracts';

import { checkIfVerified, checkIfRequested, awaitPuzzle } from '../../contracts/sms-verification';
import { postToServer } from '../../3rdparty/sms-verification';
import checkIfTxFailed from '../../util/check-if-tx-failed';
import waitForConfirmations from '../../util/wait-for-block-confirmations';
import isTestnet from '../../util/is-testnet';

const validCode = /^[A-Z\s]+$/i;

export const LOADING = 'fetching-contract';
export const QUERY_DATA = 'query-data';
export const POSTING_REQUEST = 'posting-request';
export const POSTED_REQUEST = 'posted-request';
export const REQUESTING_SMS = 'requesting-sms';
export const QUERY_CODE = 'query-code';
export const POSTING_CONFIRMATION = 'posting-confirmation';
export const POSTED_CONFIRMATION = 'posted-confirmation';
export const DONE = 'done';

export default class VerificationStore {
  @observable step = null;
  @observable error = null;

  @observable contract = null;
  @observable fee = null;
  @observable isVerified = null;
  @observable hasRequested = null;
  @observable consentGiven = false;
  @observable number = '';
  @observable requestTx = null;
  @observable code = '';
  @observable confirmationTx = null;

  @computed get isCodeValid () {
    return validCode.test(this.code);
  }
  @computed get isNumberValid () {
    return phone.isValidNumber(this.number);
  }

  @computed get isStepValid () {
    if (this.step === DONE) {
      return true;
    }
    if (this.error) {
      return false;
    }

    switch (this.step) {
      case LOADING:
        return this.contract && this.fee && this.isVerified !== null && this.hasRequested !== null;
      case QUERY_DATA:
        return this.isNumberValid && this.consentGiven;
      case QUERY_CODE:
        return this.requestTx && this.isCodeValid === true;
      case POSTED_CONFIRMATION:
        return !!this.confirmationTx;
      default:
        return false;
    }
  }

  constructor (api, account) {
    this.api = api;
    this.account = account;

    this.step = LOADING;
    Contracts.create(api).registry.getContract('smsverification')
      .then((contract) => {
        this.contract = contract;
        this.load();
      })
      .catch((err) => {
        this.error = 'Failed to fetch the contract: ' + err.message;
      });

    autorun(() => {
      if (this.error) {
        console.error('sms verification: ' + this.error);
      }
    });
  }

  @action load = () => {
    const { contract, account } = this;
    this.step = LOADING;

    const fee = contract.instance.fee.call()
      .then((fee) => {
        this.fee = fee;
      })
      .catch((err) => {
        this.error = 'Failed to fetch the fee: ' + err.message;
      });

    const isVerified = checkIfVerified(contract, account)
      .then((isVerified) => {
        this.isVerified = isVerified;
      })
      .catch((err) => {
        this.error = 'Failed to check if verified: ' + err.message;
      });

    const hasRequested = checkIfRequested(contract, account)
      .then((txHash) => {
        this.hasRequested = !!txHash;
        if (txHash) {
          this.requestTx = txHash;
        }
      })
      .catch((err) => {
        this.error = 'Failed to check if requested: ' + err.message;
      });

    Promise
      .all([ fee, isVerified, hasRequested ])
      .then(() => {
        this.step = QUERY_DATA;
      });
  }

  @action setNumber = (number) => {
    this.number = number;
  }

  @action setConsentGiven = (consentGiven) => {
    this.consentGiven = consentGiven;
  }

  @action setCode = (code) => {
    this.code = code;
  }

  @action sendRequest = () => {
    const { api, account, contract, fee, number, hasRequested } = this;

    const request = contract.functions.find((fn) => fn.name === 'request');
    const options = { from: account, value: fee.toString() };

    let chain = Promise.resolve();
    if (!hasRequested) {
      this.step = POSTING_REQUEST;
      chain = request.estimateGas(options, [])
        .then((gas) => {
          options.gas = gas.mul(1.2).toFixed(0);
          return request.postTransaction(options, []);
        })
        .then((handle) => {
          // TODO: The "request rejected" error doesn't have any property to
          // distinguish it from other errors, so we can't give a meaningful error here.
          return api.pollMethod('parity_checkRequest', handle);
        })
        .then((txHash) => {
          this.requestTx = txHash;
          return checkIfTxFailed(api, txHash, options.gas)
            .then((hasFailed) => {
              if (hasFailed) {
                throw new Error('Transaction failed, all gas used up.');
              }
              this.step = POSTED_REQUEST;
              return waitForConfirmations(api, txHash, 1);
            });
        });
    }

    chain
      .then(() => {
        return api.parity.netChain();
      })
      .then((chain) => {
        const isTest = isTestnet(chain);

        this.step = REQUESTING_SMS;
        return postToServer({ number, address: account }, isTest);
      })
      .then(() => awaitPuzzle(api, contract, account))
      .then(() => {
        this.step = QUERY_CODE;
      })
      .catch((err) => {
        this.error = 'Failed to request a confirmation SMS: ' + err.message;
      });
  }

  @action queryCode = () => {
    this.step = QUERY_CODE;
  }

  @action sendConfirmation = () => {
    const { api, account, contract, code } = this;
    const token = sha3(code);

    const confirm = contract.functions.find((fn) => fn.name === 'confirm');
    const options = { from: account };
    const values = [ token ];

    this.step = POSTING_CONFIRMATION;
    confirm.estimateGas(options, values)
      .then((gas) => {
        options.gas = gas.mul(1.2).toFixed(0);
        return confirm.postTransaction(options, values);
      })
      .then((handle) => {
        // TODO: The "request rejected" error doesn't have any property to
        // distinguish it from other errors, so we can't give a meaningful error here.
        return api.pollMethod('parity_checkRequest', handle);
      })
      .then((txHash) => {
        this.confirmationTx = txHash;
        return checkIfTxFailed(api, txHash, options.gas)
          .then((hasFailed) => {
            if (hasFailed) {
              throw new Error('Transaction failed, all gas used up.');
            }
            this.step = POSTED_CONFIRMATION;
            return waitForConfirmations(api, txHash, 1);
          });
      })
      .then(() => {
        this.step = DONE;
      })
      .catch((err) => {
        this.error = 'Failed to send the verification code: ' + err.message;
      });
  }

  @action done = () => {
    this.step = DONE;
  }
}
