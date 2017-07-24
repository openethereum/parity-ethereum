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

import { observable, computed, action } from 'mobx';
import phone from 'phoneformat.js';

import SMSVerificationABI from '@parity/shared/contracts/abi/sms-verification.json';

import VerificationStore, { LOADING, QUERY_DATA, QUERY_CODE, POSTED_CONFIRMATION, DONE } from './store';
import { isServerRunning, hasReceivedCode, postToServer } from './sms-verification';

// name in the `BadgeReg.sol` contract
const SMS_VERIFICATION = 'smsverification';

export default class SMSVerificationStore extends VerificationStore {
  @observable number = '';

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
        return this.contract && this.fee && this.accountIsVerified !== null && this.accountHasRequested !== null;
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

  constructor (api, account, isTestnet) {
    super(api, SMSVerificationABI, SMS_VERIFICATION, account, isTestnet);
  }

  isServerRunning = () => {
    return isServerRunning(this.isTestnet);
  }

  checkIfReceivedCode = () => {
    return hasReceivedCode(this.number, this.account, this.isTestnet);
  }

  // SMS verification events don't contain the phone number, so we will have to
  // send a new request every single time. See below.
  @action setIfAbleToRequest = () => {
    this.isAbleToRequest = true;
  }

  // SMS verification `request` & `confirm` transactions and events don't contain the
  // phone number, so we will have to send a new request every single time. This may
  // cost the user more money, but given that it fails otherwise, it seems like a
  // reasonable tradeoff.
  shallSkipRequest = () => Promise.resolve(false)

  @action setNumber = (number) => {
    this.number = number;
  }

  requestCode = () => {
    const { number, account, isTestnet } = this;

    return postToServer({ number, address: account }, isTestnet);
  }
}
