// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { sha3 } from '~/api/util/sha3';

import EmailVerificationABI from '~/contracts/abi/email-verification.json';
import VerificationStore, {
  LOADING, QUERY_DATA, QUERY_CODE, POSTED_CONFIRMATION, DONE
} from './store';
import { postToServer } from '../../3rdparty/email-verification';

const EMAIL_VERIFICATION = 7; // id in the `BadgeReg.sol` contract

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
        return this.contract && this.fee && this.isVerified !== null && this.hasRequested !== null;
      case QUERY_DATA:
        return this.isEmailValid && this.consentGiven;
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

  requestValues = () => [ sha3.text(this.email) ]

  @action setEmail = (email) => {
    this.email = email;
  }

  requestCode = () => {
    const { email, account, isTestnet } = this;
    return postToServer({ email, address: account }, isTestnet);
  }
}
