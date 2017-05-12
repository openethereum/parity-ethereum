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

import { action, observable } from 'mobx';

import Contracts from '~/contracts';

import { api } from '../parity.js';

let instance;

export default class ApplicationStore {
  @observable fee = null;
  @observable loading = false;
  @observable owner = null;

  accounts = [];
  api = api;
  contract = null;

  constructor () {
    this.fetchAccounts();
    this.fetchContract();
  }

  static get () {
    if (!instance) {
      instance = new ApplicationStore();
    }

    return instance;
  }

  fetchAccounts () {
    return this.api.parity
      .accountsInfo()
      .then((accountsInfo) => {
        const accounts = Object
          .keys(accountsInfo)
          .map((address) => ({
            ...accountsInfo[address],
            address
          }));

        this.accounts = accounts;
      })
      .catch((error) => {
        console.error('fetching accounts', error);
      });
  }

  fetchContract () {
    this.setLoading(true);

    return Contracts.create(this.api).registry
      .fetchContract()
      .then((contract) => {
        this.setContract(contract);

        const fee = this.fetchFee();
        const owner = this.fetchOwner();

        return Promise.all([ fee, owner ]);
      })
      .catch((error) => {
        console.error('could not fetch contract', error);
      })
      .then(() => {
        this.setLoading(false);
      });
  }

  fetchFee () {
    if (!this.contract) {
      return;
    }

    return this.contract.instance.fee.call()
      .then((fee) => this.setFee(fee))
      .catch((error) => {
        console.error('could not fetch fee', error);
      });
  }

  fetchOwner () {
    if (!this.contract) {
      return;
    }

    return this.contract.instance.owner.call()
      .then((owner) => this.setOwner(owner))
      .catch((error) => {
        console.error('could not fetch owner', error);
      });
  }

  setContract (contract) {
    this.contract = contract;
  }

  @action
  setFee (fee) {
    this.fee = fee;
  }

  @action
  setLoading (loading) {
    this.loading = loading;
  }

  @action
  setOwner (owner) {
    this.owner = owner;
  }
}
