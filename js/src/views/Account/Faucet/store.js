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

import { action, computed, observable, transaction } from 'mobx';

import apiutil from '@parity/api/util';

const ENDPOINT = 'http://faucet.kovan.network/api/';

export default class Store {
  @observable addressReceive = null;
  @observable addressVerified = null;
  @observable error = null;
  @observable responseText = null;
  @observable responseTxHash = null;
  @observable isBusy = false;
  @observable isCompleted = false;
  @observable isDestination = false;
  @observable isDone = false;

  constructor (netVersion, address) {
    transaction(() => {
      this.setDestination(netVersion === '42');

      this.setAddressReceive(address);
      this.setAddressVerified(address);
    });
  }

  @computed get canTransact () {
    return !this.isBusy && this.addressReceiveValid && this.addressVerifiedValid;
  }

  @computed get addressReceiveValid () {
    return apiutil.isAddressValid(this.addressReceive);
  }

  @computed get addressVerifiedValid () {
    return apiutil.isAddressValid(this.addressVerified);
  }

  @action setAddressReceive = (address) => {
    this.addressReceive = address;
  }

  @action setAddressVerified = (address) => {
    this.addressVerified = address;
  }

  @action setBusy = (isBusy) => {
    this.isBusy = isBusy;
  }

  @action setCompleted = (isCompleted) => {
    transaction(() => {
      this.setBusy(false);
      this.isCompleted = isCompleted;
    });
  }

  @action setDestination = (isDestination) => {
    this.isDestination = isDestination;
  }

  @action setError = (error) => {
    if (error.indexOf('not certified') !== -1) {
      this.error = `${error}. Please ensure that this account is sms certified on the mainnet.`;
    } else {
      this.error = error;
    }
  }

  @action setResponse = (response) => {
    this.responseText = response.result;
    this.responseTxHash = response.tx;
  }

  makeItRain = () => {
    this.setBusy(true);

    const options = {
      method: 'GET',
      mode: 'cors'
    };
    const url = `${ENDPOINT}${this.addressVerified}`;

    return fetch(url, options)
      .then((response) => {
        if (!response.ok) {
          return null;
        }

        return response.json();
      })
      .catch(() => {
        return null;
      })
      .then((response) => {
        transaction(() => {
          if (!response || response.error) {
            this.setError(
              response
                ? response.error
                : 'Unable to complete request to the faucet, the server may be unavailable. Please try again later.'
            );
          } else {
            this.setResponse(response);
          }

          this.setCompleted(true);
        });
      });
  }
}
