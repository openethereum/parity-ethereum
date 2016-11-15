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

import ABI from '../../contracts/abi/sms-verification.json';
// TODO: move this to a better place
const contract = '0xcE381B876A85A72303f7cA7b3a012f58F4CEEEeB';

import checkIfVerified from './check-if-verified';
import checkIfRequested from './check-if-requested';
import waitForConfirmations from './wait-for-confirmations';
import postToVerificationServer from './post-to-verification-server';

const validCode = /^[A-Z0-9_-]{7,14}$/i;

export default class VerificationStore {
  static GATHERING_DATA = 'gathering-data';
  static GATHERED_DATA = 'gathered-data';
  static POSTING_REQUEST = 'posting-request';
  static POSTED_REQUEST = 'posted-request';
  static REQUESTING_SMS = 'requesting-sms';
  static REQUESTED_SMS = 'requested-sms';
  static QUERY_CODE = 'query-code';
  static POSTING_CONFIRMATION = 'posting-confirmation';
  static POSTED_CONFIRMATION = 'posted-confirmation';
  static DONE = 'done';

  @observable step = null;
  @observable error = null;

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
    if (this.step === VerificationStore.DONE) {
      return true;
    }
    if (this.error) {
      return false;
    }

    if (this.step === VerificationStore.GATHERED_DATA) {
      return this.fee && this.isVerified === false && this.isNumberValid && this.consentGiven;
    }
    if (this.step === VerificationStore.REQUESTED_SMS) {
      return this.requestTx;
    }
    if (this.step === VerificationStore.QUERY_CODE) {
      return this.isCodeValid;
    }
    if (this.step === VerificationStore.POSTED_CONFIRMATION) {
      return this.confirmationTx;
    }
    return false;
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

  constructor (api, account) {
    this.api = api;
    this.account = account;
    this.contract = api.newContract(ABI, contract);

    autorun(() => {
      if (this.error) {
        console.error('sms verification: ' + this.error);
      }
    });
  }

  @action gatherData = () => {
    const { contract, account } = this;
    this.step = VerificationStore.GATHERING_DATA;

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
      .then((hasRequested) => {
        this.hasRequested = hasRequested;
      })
      .catch((err) => {
        this.error = 'Failed to check if requested: ' + err.message;
      });

    Promise.all([ fee, isVerified, hasRequested ])
    .then(() => {
      this.step = VerificationStore.GATHERED_DATA;
    });
  }

  @action sendRequest = () => {
    const { api, account, contract, fee, number, hasRequested } = this;

    const request = contract.functions.find((fn) => fn.name === 'request');
    const options = { from: account, value: fee.toString() };

    let chain = Promise.resolve();
    if (!hasRequested) {
      this.step = VerificationStore.POSTING_REQUEST;
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
          this.step = VerificationStore.POSTED_REQUEST;
          return waitForConfirmations(api, txHash, 1);
        });
    }

    chain
      .then(() => {
        this.step = VerificationStore.REQUESTING_SMS;
        return postToVerificationServer({ number, address: account });
      })
      .then(() => {
        this.step = VerificationStore.REQUESTED_SMS;
      })
      .catch((err) => {
        this.error = 'Failed to request a confirmation SMS: ' + err.message;
      });
  }

  @action queryCode = () => {
    this.step = VerificationStore.QUERY_CODE;
  }

  @action sendConfirmation = () => {
    const { api, account, contract, code } = this;
    const token = sha3(code);

    const confirm = contract.functions.find((fn) => fn.name === 'confirm');
    const options = { from: account };
    const values = [ token ];

    this.step = VerificationStore.POSTING_CONFIRMATION;
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
        this.step = VerificationStore.POSTED_CONFIRMATION;
        return waitForConfirmations(api, txHash, 2);
      })
      .then(() => {
        this.step = VerificationStore.DONE;
      })
      .catch((err) => {
        this.error = 'Failed to send the verification code: ' + err.message;
      });
  }

  @action done = () => {
    this.step = VerificationStore.DONE;
  }
}
