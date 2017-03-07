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
import React from 'react';
import { FormattedMessage } from 'react-intl';
import apiutil from '~/api/util';

import { txLink } from '~/3rdparty/etherscan/links';
import ShortenedHash from '~/ui/ShortenedHash';

const ENDPOINT = 'http://faucet.kovan.network/';
const KOVANTX = 'https://kovan.etherscan.io/tx/';

export default class Store {
  @observable addressReceive = null;
  @observable addressVerified = null;
  @observable error = null;
  @observable response = null;
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
    this.error = error;
  }

  @action setResponse = (response) => {
    // TODO: One the sms-faucet has an JSON API endpoint, can clean this up without parsing
    if (response.indexOf(KOVANTX) !== -1) {
      this.response = response.split(' ').map((part, index) => {
        if (part.indexOf(KOVANTX) === 0) {
          const hash = part.substr(KOVANTX.length);

          return (
            <a key='hash' href={ txLink(hash, false, '42') } target='_blank'>
              <ShortenedHash data={ hash } />
            </a>
          );
        }

        return (
          <FormattedMessage
            key={ `response_${index}` }
            id='faucet.response.part'
            defaultMessage='{part} '
            values={ {
              part
            } }
          />
        );
      });
    } else {
      this.response = response;
    }
  }

  makeItRain = () => {
    this.setBusy(true);

    // TODO: Cors not enabled atm, only opacque response
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

        // TODO: Would prefer JSON responses from the server (endpoint to be added)
        return response.text();
      })
      .catch(() => {
        return null;
      })
      .then((response) => {
        transaction(() => {
          if (response) {
            this.setResponse(response);
          } else {
            this.setError(
              <FormattedMessage
                id='faucet.error.server'
                defaultMessage='Unable to complete request to the faucet, the server may be unavailable. Please try again later.'
              />
            );
          }

          this.setCompleted(true);
        });
      });
  }
}
