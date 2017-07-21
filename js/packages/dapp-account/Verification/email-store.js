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

import { bytesToHex } from '@parity/api/util/format';
import { sha3 } from '@parity/api/util/sha3';
import EmailVerificationABI from '@parity/shared/contracts/abi/email-verification.json';

import VerificationStore, {
  LOADING, QUERY_DATA, QUERY_CODE, POSTED_CONFIRMATION, DONE
} from './store';
import { isServerRunning, hasReceivedCode, postToServer } from './email-verification';

const ZERO20 = '0x0000000000000000000000000000000000000000';

// name in the `BadgeReg.sol` contract
const EMAIL_VERIFICATION = 'emailverification';

export default class EmailVerificationStore extends VerificationStore {
  @observable email = '';

  @computed get isEmailValid () {
    // See https://davidcel.is/posts/stop-validating-email-addresses-with-regex/
    return this.email && this.email.indexOf('@') >= 0;
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
        return this.isEmailValid && this.consentGiven && this.isAbleToRequest === true;
      case QUERY_CODE:
        return this.requestTx && this.isCodeValid === true;
      case POSTED_CONFIRMATION:
        return !!this.confirmationTx;
      default:
        return false;
    }
  }

  constructor (api, account, isTestnet) {
    super(api, EmailVerificationABI, EMAIL_VERIFICATION, account, isTestnet);
  }

  isServerRunning = () => {
    return isServerRunning(this.isTestnet);
  }

  checkIfReceivedCode = () => {
    return hasReceivedCode(this.email, this.account, this.isTestnet);
  }

  // If the email has already been used for verification of another account,
  // we prevent the user from wasting ETH to request another verification.
  @action setIfAbleToRequest = () => {
    const { isEmailValid } = this;

    if (!isEmailValid) {
      this.isAbleToRequest = true;
      return;
    }

    const { contract, email } = this;
    const emailHash = sha3.text(email);

    this.isAbleToRequest = null;
    contract
      .instance.reverse
      .call({}, [ emailHash ])
      .then((address) => {
        if (address === ZERO20) {
          this.isAbleToRequest = true;
        } else {
          this.isAbleToRequest = new Error('Another account has been verified using this e-mail.');
        }
      })
      .catch((err) => {
        this.error = 'Failed to check if able to send request: ' + err.message;
      });
  }

  // Determine the values relevant for checking if the last request contains
  // the same data as the current one.
  requestValues = () => [ sha3.text(this.email) ]

  shallSkipRequest = (currentValues) => {
    const { accountHasRequested } = this;
    const lastRequest = this.lastRequestValues;

    if (!accountHasRequested) {
      return Promise.resolve(false);
    }
    // If the last email verification `request` for the selected address contains
    // the same email as the current one, don't send another request to save ETH.
    const skip = currentValues[0] === bytesToHex(lastRequest.emailHash.value);

    return Promise.resolve(skip);
  }

  @action setEmail = (email) => {
    this.email = email;
  }

  requestCode = () => {
    const { email, account, isTestnet } = this;

    return postToServer({ email, address: account }, isTestnet);
  }
}
